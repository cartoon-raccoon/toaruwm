//! This module defines `Workspace`, which represents a collection
//! of windows that can be displayed onscreen together.
//! Each workspace tracks its windows inside a `ClientRing`,
//! and also tracks whether each window is tiled or floating.
//! Workspace layouts can be changed on the fly, and will adjust themselves
//! as necessary.

use std::fmt;

use tracing::instrument;
use tracing::{debug, error, trace, warn};

use crate::core::{
    desktop::Screen,
    window::{Client, ClientRing},
};
use crate::layouts::{LayoutAction, LayoutEngine, LayoutFn, LayoutType};
use crate::types::{BorderStyle, ClientAttrs, ClientConfig, Direction, BORDER_WIDTH};
use crate::x::{core::StackMode, XConn, XWindowID};
use crate::Result;

/// A grouped collection of windows arranged according to a Layout.
///
/// Workspaces are managed as a group by a Desktop.
#[derive(Clone)]
pub struct Workspace {
    pub(crate) name: String,
    pub(crate) windows: ClientRing,
    pub(crate) master: Option<XWindowID>,
    pub(crate) layoutter: LayoutEngine,
}

impl fmt::Debug for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Workspace")
            .field("name", &self.name)
            .field("windows", &self.windows)
            .field("master", &self.master)
            .field("layout", &self.layoutter.layout())
            .finish()
    }
}

impl Workspace {
    /// Creates a new workspace with a specific layout.
    pub fn with_layout(layout: LayoutType, lfn: Option<LayoutFn>, name: &str) -> Self {
        Self {
            name: name.into(),
            windows: ClientRing::new(),
            master: None,
            layoutter: LayoutEngine::with_layout(layout, lfn),
        }
    }

    /// Sets the layout to use and applies it to all currently mapped windows.
    pub fn set_layout<X: XConn>(
        &mut self,
        layout: LayoutType,
        lfn: Option<LayoutFn>,
        conn: &X,
        scr: &Screen,
    ) {
        self.layoutter.set_layout(layout, lfn);
        self.relayout(conn, scr);
    }

    /// Tests whether the workspace contains a specfic window.
    pub fn contains_window(&self, id: XWindowID) -> bool {
        self.windows.contains(id)
    }

    /// Returns a reference to the currently focused client.
    pub fn focused_client(&self) -> Option<&Client> {
        self.windows.focused()
    }

    /// Returns a mutable reference to the currently focused client.`
    pub fn focused_client_mut(&mut self) -> Option<&mut Client> {
        self.windows.focused_mut()
    }

    ///  Tests whether `id` is the master window.
    #[inline(always)]
    pub fn is_master(&mut self, id: XWindowID) -> bool {
        let Some(win) = self.master else {return false};
        win == id
    }

    /// Returns the name of the workspace.
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the id of the master window.
    #[inline(always)]
    pub fn master(&self) -> Option<XWindowID> {
        self.master
    }

    /// Returns an iterator over all the clients in the workspace.
    #[inline]
    pub fn clients(&self) -> impl Iterator<Item = &Client> {
        self.windows.iter()
    }

    /// Returns a mutable iterator over all the clients in the workspace.
    #[inline]
    pub fn clients_mut(&mut self) -> impl Iterator<Item = &mut Client> {
        self.windows.iter_mut()
    }

    /// Returns the number of tiled windows in the workspace.
    ///
    /// Since a workspace can contain both floating and tiled windows,
    /// this returns the number of tiled windows only.
    pub fn tiled_count(&self) -> usize {
        self.windows.iter().filter(|win| win.is_tiled()).count()
    }

    /// Returns the number of floating windows in the workspace.
    ///
    /// Since a workspace can contain both floating and tiled windows,
    /// this returns the number of floating windows only.
    pub fn floating_count(&self) -> usize {
        self.windows.iter().filter(|win| win.is_floating()).count()
    }

    /// Tests whether the workspace is empty.`
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Tests whether the workspace is tiled.
    #[inline(always)]
    pub fn is_tiling(&self) -> bool {
        !self.is_floating()
    }

    /// Tests whether the workspace is floating.
    #[inline(always)]
    pub fn is_floating(&self) -> bool {
        matches!(self.layoutter.layout(), LayoutType::Floating)
    }

    /// Returns the layout type of the workspace.
    #[inline]
    pub fn layout(&self) -> &LayoutType {
        self.layoutter.layout()
    }

    /// Returns the `Some(idx)` where `idx` is the index of the
    /// Client in its underlying ring, or `None` if the Client
    /// does not exist.
    #[inline]
    pub fn contains(&self, window: XWindowID) -> Option<usize> {
        self.windows.get_idx(window)
    }

    /// Sets the master window of the workspace.
    pub fn set_master(&mut self, master_id: XWindowID) {
        // either we have no tiled windows
        // or we're adding a window that we're not supposed to
        if !self.windows.contains(master_id) {
            if self.tiled_count() == 0 {
                self.master = Some(master_id);
            } else {
                error!("set_master: No such window {}", master_id);
            }
            return;
        }
        self.master = Some(master_id);
        let idx = self.windows.get_idx(master_id).unwrap();
        self.windows.move_front(idx);
    }

    /// Unsets the master window of the workspace.
    ///
    /// Is a no-op if there are still tiled windows in the workspace.
    pub fn unset_master(&mut self) {
        if self.tiled_count() > 0 {
            error!("unset_master: Workspace still has tiled windows");
            return;
        }
        self.master = None;
    }

    /// Maps all the windows in the workspace.
    ///
    /// The window that gets the focus in the one that is currently
    /// focused in the internal Ring.
    #[instrument(level = "debug", skip(self, conn))]
    pub fn activate<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        if self.windows.is_empty() {
            return;
        }

        //todo: change this to account for all layouts
        //* currently does not re-apply layouts when done

        self.relayout(conn, scr);

        // focus the main window in the workspace
        // if floating, focus the first window
        // else (should be tiled), focus the master window
        if self.is_floating() {
            assert!(self.master.is_none());
            if !self.is_empty() {
                self.focus_window(conn, self.windows.get(0).unwrap().id());
            }
        } else {
            trace!("master is {:?}", self.master);
            if let Some(win) = self.master {
                self.focus_window(conn, win);
            }
        }

        for window in self.windows.iter_rev() {
            // disable events
            window.change_attributes(conn, &[ClientAttrs::DisableClientEvents]);
            // update window geometry in the x server
            //? is this necessary, since Self::relayout already updates the geom
            window.update_geometry(conn);
            // map window
            conn.map_window(window.id())
                .unwrap_or_else(|e| error!("{}", e));
            // re-enable events
            window.change_attributes(conn, &[ClientAttrs::EnableClientEvents]);
        }
    }

    /// Unmaps all the windows in the workspace.
    #[instrument(level = "debug", skip(self, conn))]
    pub fn deactivate<X: XConn>(&mut self, conn: &X) {
        for window in self.windows.iter() {
            conn.change_window_attributes(window.id(), &[ClientAttrs::DisableClientEvents])
                .unwrap_or_else(|e| error!("{}", e));

            conn.unmap_window(window.id())
                .unwrap_or_else(|e| error!("{}", e));

            conn.change_window_attributes(window.id(), &[ClientAttrs::EnableClientEvents])
                .unwrap_or_else(|e| error!("{}", e));
        }
    }

    /// Calls the layout function and applies it to the workspace.
    pub fn relayout<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        let layouts = self.layoutter.gen_layout(self, scr);
        self.apply_layout(conn, layouts);
    }

    /// Adds a new window and maps it.
    pub fn add_window<X: XConn>(&mut self, conn: &X, scr: &Screen, id: XWindowID) {
        match self.layout() {
            LayoutType::Floating => {
                self.add_window_floating(conn, scr, id);
            }
            _ => self.add_window_tiled(conn, scr, id),
        }
    }

    /// Deletes the window from the workspaces and returns it.
    #[instrument(level = "debug", skip(self, conn, scr))]
    pub fn del_window<X: XConn>(
        &mut self,
        conn: &X,
        scr: &Screen,
        id: XWindowID,
    ) -> Result<Option<Client>> {
        //todo: make all workspace methods return Result
        if let Some(win) = self.windows.lookup(id) {
            if win.is_floating() {
                Ok(Some(self.del_window_floating(conn, scr, id)))
            } else {
                Ok(Some(self.del_window_tiled(conn, scr, id)))
            }
        } else {
            // fail silently (this accounts for spurious unmap events)
            Ok(None)
        }
    }

    pub(crate) fn add_window_floating<X: XConn>(&mut self, conn: &X, scr: &Screen, id: XWindowID) {
        self._add_window(conn, scr, Client::floating(id, conn));
    }

    pub(crate) fn add_window_tiled<X: XConn>(&mut self, conn: &X, scr: &Screen, id: XWindowID) {
        if self.master.is_none() {
            self.set_master(id);
        }
        self._add_window(conn, scr, Client::tiled(id, conn));
    }

    #[instrument(level = "debug", skip(self, conn, scr, window))]
    fn _add_window<X: XConn>(&mut self, conn: &X, scr: &Screen, mut window: Client) {
        trace!("adding window {:#?}", window);
        // Set supported protocols
        window.set_supported(conn);
        // Configure window with a border width
        window.configure(conn, &[ClientConfig::BorderWidth(BORDER_WIDTH)]);

        // // if there is a focused window, stack this window above
        // if self.windows.focused().is_some() {
        //     window.configure(conn, &[ClientConfig::StackingMode(StackMode::Above)]);
        // }

        // // set the internal geometry
        // //? is this line necessary?
        // window.xwindow.set_geometry_conn(conn);

        // add the window to internal client storage
        let id = window.id();
        self.windows.append(window);

        // enable client events on the window
        conn.change_window_attributes(id, &[ClientAttrs::EnableClientEvents])
            .unwrap_or_else(|e| error!("change window attributes failed: {}", e));

        // apply the relevant layout to the screen
        // this also internally updates the geometries on the server
        // as well as locally
        self.relayout(conn, scr);

        // map window
        self.windows.lookup_mut(id).unwrap().map(conn);

        // set input focus
        if let Some(curr_focused) = self.windows.focused_mut() {
            let to_unfocus = curr_focused.id();
            self.unfocus_window(conn, to_unfocus);
        }

        self.focus_window(conn, id);
    }

    fn del_window_floating<X: XConn>(&mut self, conn: &X, scr: &Screen, id: XWindowID) -> Client {
        let window = self
            .windows
            .remove_by_id(id)
            //todo: return Result instead
            .expect("Could not find window");

        if let Some(win) = self.windows.focused() {
            self.stack_and_focus_window(conn, win.id());
        }

        if self.is_empty() {
            self.windows.unset_focused();
        }

        self.relayout(conn, scr);

        window
    }

    fn del_window_tiled<X: XConn>(&mut self, conn: &X, scr: &Screen, id: XWindowID) -> Client {
        // internally remove window from tracking
        let window = self
            .windows
            .remove_by_id(id)
            .expect("Could not find window");

        // set new workspace master or unset it if empty
        if self.is_master(id) {
            debug!("window to destroy is master, doing unmap checks");
            if self.tiled_count() == 0 {
                // workspace is empty
                debug!("workspace is now empty, unsetting master");
                self.unset_master(); //workspace is now empty
                self.windows.unset_focused();
            } else {
                // workspace is not empty, so set new master
                debug!(
                    "workspace has {} tiled windows, setting new master",
                    self.tiled_count()
                );
                // get the next window in our client ring
                // safe to unwrap since we have at least one element
                let new_master = self.windows.get(0).unwrap().id();
                // set new master internally
                debug!("New master is now {}", new_master);
                self.set_master(new_master);
                debug!("Window at idx 0 is {:#?}", self.windows.get(0));
                self.focus_window(conn, new_master);
            }
        } else {
            // the deleted window was not the master
            debug!("window to destroy is not master, doing unmap checks");
            if self.tiled_count() == 1 {
                // only master is left
                debug!("only master window remaining");
                let master = self.master.unwrap();
                self.focus_window(conn, master);
            } else if !self.is_empty() {
                // no tiled, but may have floating
                assert!(self.tiled_count() >= 1 || self.floating_count() >= 1);

                // set focus to master if we have one,
                // else set focus to next available window
                let to_focus = if let Some(master) = self.master {
                    master
                } else {
                    self.windows.get(0).unwrap().id()
                };
                self.focus_window(conn, to_focus);
            } else {
                error!("this branch should be unreachable");
                self.windows.unset_focused();
            }
        }

        // recalculate layouts
        self.relayout(conn, scr);

        window
    }

    /// Pushes a window directly.
    pub(crate) fn push_window(&mut self, window: Client) {
        if let LayoutType::Floating = self.layout() {
            self.windows.push(window);
        } else if self.master.is_none() {
            if self.tiled_count() > 0 {
                warn!("Windows not empty but workspace has no master")
            }
            if window.is_tiled() {
                let window_id = window.id();
                self.windows.push(window);
                self.set_master(window_id);
            } else {
                self.windows.push(window);
            }
        } else {
            self.windows.append(window);
        }
    }

    fn apply_layout<X: XConn>(&mut self, conn: &X, layouts: Vec<LayoutAction>) {
        // get all floating windows and stack them above all tiled
        for floater in self.clients_mut().filter(|c| c.is_floating()) {
            floater.configure(conn, &[ClientConfig::StackingMode(StackMode::Above)])
        }

        for rsaction in layouts {
            match rsaction {
                LayoutAction::SetMaster(id) => {
                    let master_idx = self.windows.get_idx(id).unwrap();
                    self.windows.move_front(master_idx);
                    self.set_master(id);
                }
                LayoutAction::UnsetMaster => self.unset_master(),
                LayoutAction::Resize { id, geom } => {
                    let window = self.windows.lookup_mut(id).unwrap();
                    window.set_and_update_geometry(conn, geom);
                }
            }
        }
    }

    /// Sets the input focus, internally and on the server, to the given ID.
    //#[instrument(level = "debug", skip(self, conn))]
    pub fn focus_window<X: XConn>(&mut self, conn: &X, window: XWindowID) {
        let Some(_) = self.windows.get_idx(window) else {
            warn!("focus_window: no window {} found in workspace", window);
            return
        };

        debug!("found window {}", window);

        // unfocus the current focused window
        if let Some(focused) = self.windows.focused_mut() {
            let id = focused.id();
            self.unfocus_window(conn, id);
        }
        // focus the window
        self.stack_and_focus_window(conn, window);
    }

    /// Unfocuses the given window ID.
    pub fn unfocus_window<X: XConn>(&mut self, conn: &X, window: XWindowID) {
        // remove focus if window to unfocus is currently focused
        if self.windows.lookup(window).is_some() {
            conn.change_window_attributes(
                window,
                &[ClientAttrs::BorderColour(BorderStyle::Unfocused)],
            )
            .unwrap_or_else(|e| error!("{}", e));
        } else {
            warn!("no such window {} to unfocus", window)
        }
    }

    /// Convenience function that does the following:
    ///
    /// - Stacks the given window above.
    /// - Sets the input focus to it.
    ///
    /// Note: THE WINDOW MUST EXIST.
    fn stack_and_focus_window<X: XConn>(&mut self, conn: &X, window: XWindowID) {
        use BorderStyle::*;
        // disable events
        conn.change_window_attributes(window, &[ClientAttrs::DisableClientEvents])
            .unwrap_or_else(|e| error!("{}", e));

        let win = self.windows.lookup_mut(window).unwrap();

        // if there is a focused window, stack it above
        win.configure(conn, &[ClientConfig::StackingMode(StackMode::Above)]);

        // focus to current window
        win.set_border(conn, Focused);
        conn.set_input_focus(window);
        self.windows.set_focused_by_winid(window);

        // re-enable events
        conn.change_window_attributes(window, &[ClientAttrs::EnableClientEvents])
            .unwrap_or_else(|e| error!("{}", e));
    }

    /// Cycles the focus to the next window in the workspace.
    pub fn cycle_focus<X: XConn>(&mut self, conn: &X, dir: Direction) {
        use BorderStyle::*;

        // change focus colours
        let win = if let Some(win) = self.windows.focused() {
            win.id()
        } else {
            error!("cycle_focus for ws {}: nothing focused", self.name);
            return;
        };

        //change currently focused border colour to unfocused
        if let Some(win) = self.windows.focused_mut() {
            win.set_border(conn, Unfocused);
        }

        //internally, cycle focus
        self.windows.cycle_focus(dir);

        self.focus_window(conn, win);
    }

    /// Cycles the master to the next window in the workspace.
    ///
    /// Is a no-op if the layout is not tiling.
    pub fn cycle_master<X: XConn>(&mut self, conn: &X, scr: &Screen, dir: Direction) {
        if !self.is_tiling() {
            return;
        }

        if !self.windows.is_empty() {
            self.windows.rotate(dir);
            self.master = Some(self.windows.get(0).unwrap().id());
            self.relayout(conn, scr);
        }
    }

    /// Deletes the focused window in the workspace and returns it.
    pub fn take_focused_window<X: XConn>(&mut self, conn: &X, screen: &Screen) -> Option<Client> {
        if let Some(window) = self.windows.focused() {
            let id = window.id();
            self.del_window(conn, screen, id).ok()?
        } else {
            None
        }
    }

    /// Toggles the state of the currently focused window between floating and tiled.
    pub fn toggle_focused_state<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        debug!(
            "Toggling state of focused window {:#?}",
            self.windows.focused()
        );
        let master = self.master;
        // If we have a focused window
        if let Some(win) = self.windows.focused_mut() {
            // set a stack variable to avoid overlapping lifetimes
            let win_id = win.id();
            debug!("Toggling window state");
            win.toggle_state();
            if win.is_floating() {
                //toggling to tiled
                // if we have no master
                if master.is_none() {
                    debug!("No master, setting master");
                    self.set_master(win_id);
                }
                // keep floating windows on top
            } else {
                //toggling to floating
                win.configure(conn, &[ClientConfig::StackingMode(StackMode::Above)]);

                if self.tiled_count() == 0 && self.master.is_some() {
                    // if master is the only window
                    debug!("All windows are floating, unsetting master");
                    self.unset_master();
                } else if self.tiled_count() > 0 && master.is_some() {
                    // if window to toggle is master
                    if master.unwrap() == win_id {
                        debug!("Window to toggle is master, setting new master");
                        // we can get idx 1 and safely unwrap because windows.len() >= 2
                        let new_master = self.windows.get(1).expect("No window of idx 1").id();
                        self.set_master(new_master);
                    }
                } else {
                    assert!(master.is_none());
                }
            }
            self.relayout(conn, scr);
        }
    }

    /// Sets the focused window to tiled and re-applies the layout.
    ///
    /// Is a no-op if the workspace is in a floating layout.
    pub fn set_tiled<X: XConn>(&mut self, conn: &X, id: XWindowID, scr: &Screen) {
        debug!("Setting focused to tiled");

        if self.is_floating() {
            return;
        }

        let master = self.master;

        if let Some(win) = self.windows.lookup_mut(id) {
            if win.is_tiled() {
                return;
            }

            let win_id = win.id();

            win.set_tiled();

            // assuming
            if self.tiled_count() == 0 && master.is_none() {
                debug!("All windows are floating, setting master");
                self.set_master(win_id);
            }

            self.relayout(conn, scr);
        }
    }

    /// Sets the focused window to floating and re-applies the layout.
    pub fn set_floating<X: XConn>(&mut self, conn: &X, id: XWindowID, scr: &Screen) {
        debug!("Setting focused to floating");
        let master = self.master;

        if let Some(win) = self.windows.lookup_mut(id) {
            if win.is_floating() {
                return;
            }

            let win_id = win.id();

            win.set_floating();
            win.configure(conn, &[ClientConfig::StackingMode(StackMode::Above)]);
            let i = win.initial_geom();
            win.set_size(conn, i.height, i.width);

            if self.tiled_count() == 0 && master.is_some() {
                debug!("All windows are floating, unsetting master");
                self.unset_master();
            } else if self.tiled_count() > 0 && master.is_some() {
                if master.unwrap() == win_id {
                    debug!("Window to set floating is master, setting new master");

                    let new = self.windows.get(1).expect("No window of idx 1");
                    if new.is_floating() {
                        warn!(
                            "New master of id {} name {} is floating",
                            new.id(),
                            new.name()
                        );
                    }
                    let new_id = new.id();
                    self.set_master(new_id);
                }
            } else {
                assert!(master.is_none());
            }
            self.relayout(conn, scr);
        }
    }
}
