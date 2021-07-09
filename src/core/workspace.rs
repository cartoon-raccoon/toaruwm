//! This module defines `Workspace`, which represents a collection
//! of windows that can be displayed onscreen together.
//! Each workspace tracks its windows inside a `ClientRing`,
//! and also tracks whether each window is tiled or floating.
//! Workspace layouts can be changed on the fly, and will adjust themselves
//! as necessary.

use std::fmt;

use crate::{Result, ToaruError};
use crate::core::{
    window::{Client, ClientRing},
    desktop::Screen,
};
use crate::types::{
    BorderStyle, Direction,
    ClientAttrs,
    ClientConfig,
    BORDER_WIDTH,
};
use crate::layouts::{
    LayoutType, 
    LayoutEngine, 
    LayoutAction,
    LayoutFn,
};
use crate::x::{XConn, XWindowID, core::StackMode};

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
        scr: &Screen
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

    /// Maps all the windows in the workspace.
    pub fn activate<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        if self.windows.is_empty() {
            return
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
            debug!("Master is {:?}", self.master);
            if let Some(win) = self.master {
                self.focus_window(conn, win);
            }
        }

        for window in self.windows.iter_rev() {
            // disable events
            window.change_attributes(conn, &[ClientAttrs::DisableClientEvents]);
            // update window geometry in the x server
            window.update_geometry(conn);
            // map window
            conn.map_window(window.id()).unwrap_or_else(|e| error!("{}", e));
            // re-enable events
            window.change_attributes(conn, &[ClientAttrs::EnableClientEvents]);
        }
    }

    /// Unmaps all the windows in the workspace.
    pub fn deactivate<X: XConn>(&mut self, conn: &X) {
        for window in self.windows.iter() {
            conn.change_window_attributes(window.id(), &[ClientAttrs::DisableClientEvents])
            .unwrap_or_else(|e| error!("{}", e));
    
            conn.unmap_window(window.id()).unwrap_or_else(|e| error!("{}", e));
    
            conn.change_window_attributes(window.id(), &[ClientAttrs::EnableClientEvents])
            .unwrap_or_else(|e| error!("{}", e));
        }
    }

    /// Adds a new window and maps it.
    pub fn add_window<X: XConn>(&mut self, conn: &X, scr: &Screen, id: XWindowID) {
        match self.layout() {
            LayoutType::Floating => {
                self.add_window_floating(conn, scr, id);
            }
            _ => {
                self.add_window_tiled(conn, scr, id)
            }
        }
    }

    /// Deletes the window from the workspaces and returns it.
    pub fn del_window<X: XConn>(&mut self, 
        conn: &X, scr: &Screen, id: XWindowID
    ) -> Result<Client> {
        if let Some(win) = self.windows.lookup(id) {
            if win.is_floating() {
                return Ok(self.del_window_floating(conn, scr, id))
            } else {
                return Ok(self.del_window_tiled(conn, scr, id))
            }
        }
        Err(ToaruError::UnknownClient(id))
    }

    pub(crate) fn add_window_floating<X: XConn>(&mut self, 
        conn: &X, scr: &Screen, id: XWindowID
    ) {
        fn_ends!("add_window_floating");
        self._add_window(conn, scr, Client::floating(id, conn));
    }

    pub(crate) fn add_window_tiled<X: XConn>(&mut self, 
        conn: &X, scr: &Screen, id: XWindowID
    ) {
        fn_ends!("add_window_tiled");
        if self.master.is_none() {
            self.set_master(id);
        }
        self._add_window(conn, scr, Client::tiled(id, conn));
    }

    fn _add_window<X: XConn>(&mut self,
        conn: &X, scr: &Screen, mut window: Client
    ) {
        window.set_supported(conn);
        window.map(conn);
        window.configure(conn, &[
            ClientConfig::BorderWidth(BORDER_WIDTH)
        ]);

        if self.windows.focused().is_some() {
            window.configure(conn, &[ClientConfig::StackingMode(StackMode::Above)]);
        }

        window.xwindow.set_geometry_conn(conn);

        if let Ok(ptr) = conn.query_pointer(conn.get_root().id) {
            if ptr.child == conn.get_root().id || ptr.child == window.id() {
                self.focus_window(conn, window.id());
            } else if let Some(focused) = self.windows.focused_mut() {
                focused.set_border(conn, BorderStyle::Unfocused);
            } else {
                window.set_border(conn, BorderStyle::Unfocused);
            }
        }

        window.change_attributes(conn, &[ClientAttrs::EnableClientEvents]);

        self.windows.append(window);
        self.relayout(conn, scr);
    }

    #[allow(mutable_borrow_reservation_conflict)]
    fn del_window_floating<X: XConn>(&mut self, 
        conn: &X, scr: &Screen, id: XWindowID
    ) -> Client {
        let window = self.windows.remove_by_id(id)
        //todo: return Result instead
        .expect("Could not find window");

        window.change_attributes(conn, &[ClientAttrs::DisableClientEvents]);
        //window.unmap(conn);

        if let Some(win) = self.windows.focused() {
            window_stack_and_focus(self, conn, win.id());
        }

        if self.is_empty() {
            self.windows.unset_focused();
        }

        self.relayout(conn, scr);

        window
    }

    fn del_window_tiled<X: XConn>(&mut self, 
        conn: &X, scr: &Screen, id: XWindowID
    ) -> Client {
        fn_ends!("[start] dtiled::del_window");

        // internally remove window from tracking
        let window = self.windows.remove_by_id(id)
        .expect("Could not find window");

        // disable events and unmap the window
        window.change_attributes(conn, &[ClientAttrs::DisableClientEvents]);
        self.windows.unset_focused();

        // set new workspace master or unset it if empty
        if self.is_master(id) {
            debug!("del_window: Window to destroy is master, doing unmap checks");
            if self.tiled_count() == 0 {
                debug!(
                    "del_window: Workspace is now empty, unsetting master"
                );
                self.unset_master(); //workspace is now empty
                self.windows.unset_focused();
            } else {
                debug!(
                    "del_window: Workspace has {} tiled windows, setting new master", 
                    self.tiled_count()
                );
                let new_master = self.windows.get(0).unwrap().id();
                debug!("New master is now {}", new_master);
                self.set_master(new_master);
                debug!("Window at idx 0 is {:#?}", self.windows.get(0));
                window_stack_and_focus(self, conn, new_master);
            }
        } else {
            // only master is left
            if self.tiled_count() == 1 {
                let master = self.master.unwrap();
                window_stack_and_focus(self, conn, master);        
            } else if !self.is_empty() {
                assert!(self.tiled_count() > 1);
                //todo: add last focused so we can focus to that
                // placeholder code to focus to master by default
                let master = self.master.unwrap();
                window_stack_and_focus(self, conn, master);
            } else {
                self.windows.unset_focused();
            }
        }

        // recalculate layouts
        self.relayout(conn, scr);

        fn_ends!("[end] dtiled::del_window");
        window
    }

    /// Pushes a window directly.
    pub(crate) fn push_window(&mut self, window: Client) {
        fn_ends!("[start] workspace::push_window");
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
        fn_ends!("[end] workspace::push_window");
    }

    /// Calls the layout function and applies it to the workspace.
    pub fn relayout<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        let layouts = self.layoutter.gen_layout(&self, scr);
        self.apply_layout(conn, layouts);
    }

    fn apply_layout<X: XConn>(
        &mut self, 
        conn: &X, 
        layouts: Vec<LayoutAction>
    ) {
        for rsaction in layouts {
            match rsaction {
                LayoutAction::SetMaster(id) => {
                    let master_idx = self.windows.get_idx(id).unwrap();
                    self.windows.move_front(master_idx);
                    self.set_master(id);
                }
                LayoutAction::UnsetMaster => self.unset_master(),
                LayoutAction::Resize {id, geom} => {
                    let window = self.windows.lookup_mut(id).unwrap();
                    window.set_and_update_geometry(conn, geom);
                }
            }
        }
    }

    /// Sets the focused window to the given window ID.
    pub fn focus_window<X: XConn>(&mut self, conn: &X, window: XWindowID) {
        if let Some(idx) = self.windows.get_idx(window) {
            debug!("Found window {}", window);
            if let Some(focused) = self.windows.focused_mut() {
                focused.set_border(conn, BorderStyle::Unfocused);
            }
            // internally focus
            self.windows.set_focused_by_idx(idx);
            
            // tell x to focus
            window_stack_and_focus(self, conn, window);
        } else {
            warn!("No window {} found in workspace", window);
        }
    }

    /// Unfocuses the given window ID.
    pub fn unfocus_window<X: XConn>(&mut self, conn: &X, window: XWindowID) {
        // remove focus if window to unfocus is currently focused
        if let Some(_) = self.windows.lookup(window) {
            conn.change_window_attributes(window, &[
                ClientAttrs::BorderColour(BorderStyle::Unfocused)
            ]).unwrap_or_else(|e| error!("{}", e));
        }
    }

    /// Cycles the focus to the next window in the workspace.
    #[allow(mutable_borrow_reservation_conflict)]
    pub fn cycle_focus<X: XConn>(&mut self, conn: &X, dir: Direction) {
        use BorderStyle::*;

        //change currently focused border colour to unfocused
        if let Some(win) = self.windows.focused_mut() {
            win.set_border(conn, Unfocused);
        }
        
        //internally, cycle focus
        self.windows.cycle_focus(dir);

        // change focus colours
        if let Some(win) = self.windows.focused() {
            self.focus_window(conn, win.id());
        }
    }

    /// Cycles the master to the next window in the workspace.
    pub fn cycle_master<X: XConn>(&mut self, 
        conn: &X, scr: &Screen, dir: Direction
    ) {
        if !self.is_tiling() {return}

        if !self.windows.is_empty() {
            self.windows.rotate(dir);
            self.master = Some(self.windows.get(0).unwrap().id());
            self.relayout(conn, scr);
        }
    }

    /// Deletes the focused window in the workspace and returns it.
    pub fn take_focused_window<X: XConn>(&mut self,
        conn: &X, screen: &Screen,
    ) -> Option<Client> {
        if let Some(window) = self.windows.focused() {
            let window = window.to_owned();
            self.del_window(conn, screen, window.id()).ok()?;

            Some(window)
        } else {
            None
        }
    }

    /// Toggles the state of the currently focused window between floating and tiled.
    pub fn toggle_focused_state<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        debug!("Toggling state of focused window {:#?}", self.windows.focused());
        let master = self.master;
        // If we have a focused window
        if let Some(win) = self.windows.focused_mut() {
            // set a stack variable to avoid overlapping lifetimes
            let win_id = win.id();
            debug!("Toggling window state");
            win.toggle_state();
            if win.is_floating() { //toggling to tiled
                // if we have no master
                if master.is_none() {
                    debug!("No master, setting master");
                    self.set_master(win_id);
                }
                // keep floating windows on top
            } else { //toggling to floating
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
    pub fn set_focused_tiled<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        debug!("Setting focused to tiled");

        if self.is_floating() {return}

        let master = self.master;

        if let Some(win) = self.windows.focused_mut() {
            if win.is_tiled() {return}

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
    pub fn set_focused_floating<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        debug!("Setting focused to floating");
        let master = self.master;

        if let Some(win) = self.windows.focused_mut() {
            if win.is_floating() {return}

            let win_id = win.id();

            win.set_floating();
            win.configure(conn, &[ClientConfig::StackingMode(StackMode::Above)]);

            if self.tiled_count() == 0 && master.is_some() {
                debug!("All windows are floating, unsetting master");
                self.unset_master();
            } else if self.tiled_count() > 0 && master.is_some() {
                if master.unwrap() == win_id {
                    debug!("Window to set floating is master, setting new master");

                    let new_master = self.windows.get(1).expect("No window of idx 1").id();
                    self.set_master(new_master);
                }
            } else {
                assert!(master.is_none());
            }
            self.relayout(conn, scr);
        }
    }

    /// Sets the master window of the workspace.
    pub fn set_master(&mut self, master_id: XWindowID) {
        if !self.windows.contains(master_id) {
            error!("set_master: No such window {}", master_id);
            return
        }
        self.master = Some(master_id);
        let idx = self.windows.get_idx(master_id).unwrap();
        self.windows.move_front(idx);
    }

    /// Unsets the master window of the workspace.
    pub fn unset_master(&mut self) {
        if self.tiled_count() > 0 {
            error!("unset_master: Workspace still has tiled windows");
        }
        self.master = None;
    }

    ///  Tests whether `id` is the master window.
    #[inline(always)]
    pub fn is_master(&mut self, id: XWindowID) -> bool {
        if let Some(win) = self.master {
            return win == id
        }
        false
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

    #[inline]
    pub fn contains(&self, window: XWindowID) -> Option<usize> {
        self.windows.get_idx(window)
    }
}

fn window_stack_and_focus<X: XConn>(ws: &mut Workspace, conn: &X, window: XWindowID) {
    use BorderStyle::*;
    // disable events
    conn.change_window_attributes(window, &[ClientAttrs::DisableClientEvents])
    .unwrap_or_else(|e| error!("{}", e));

    let win = ws.windows.lookup_mut(window).unwrap();

    // if there is a focused window, stack it above

    win.configure(conn, &[ClientConfig::StackingMode(StackMode::Above)]);
    
    // focus to current window
    win.set_border(conn, Focused);
    conn.set_input_focus(window);

    // re-enable events
    conn.change_window_attributes(window, &[ClientAttrs::EnableClientEvents])
    .unwrap_or_else(|e| error!("{}", e));
}