//! This module defines `Workspace`, which represents a collection
//! of windows that can be displayed onscreen together, with a set
//! of layouts that can be swapped out or modified on the fly.

use std::fmt;

use tracing::instrument;
use tracing::{debug, error, trace, warn};

use crate::core::{
    desktop::Screen,
    window::{Client, ClientRing},
    ring::Ring,
};
use crate::layouts::{
    Layout, Layouts, LayoutAction, LayoutType,
    update::IntoUpdate,
};
use crate::types::{BorderStyle, ClientAttrs, ClientConfig, Direction, BORDER_WIDTH};
use crate::x::{core::StackMode, XConn, XWindowID};
use crate::Result;

/// A specification describing a workspace.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceSpec {
    pub(crate) name: String,
    pub(crate) idx: usize,
    pub(crate) layouts: Vec<String>,
}

impl WorkspaceSpec {
    /// Creates a new WorkspaceSpec.
    pub fn new<S, L>(name: S, screen: usize, layouts: L) -> Self
    where
        S: Into<String>,
        L: IntoIterator<Item = String>
    {
        Self {
            name: name.into(),
            idx: screen,
            layouts: layouts.into_iter().collect(),
        }
    }

    /// Getter method returning the specification's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Getter method returning the specification's screen.
    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Getter method returning the specification's layouts.
    pub fn layouts(&self) -> &[String] {
        &self.layouts
    }
}

/// A grouped collection of windows arranged according to a Layout.
/// 
/// ## General Usage
/// 
/// Workspaces manage windows in two classes; within the layout
/// or outside the layout, marked by an attribute on the window.
/// windows within the layout are passed to the layout generator
/// to account for when it generates the layout, and windows outside
/// of the layout are always stacked on top and floated.
/// 
/// Most Workspace methods involve adding or removing windows, swapping
/// layouts, or modifying layouts.
/// 
pub struct Workspace {
    pub(crate) name: String,
    pub(crate) windows: ClientRing,
    pub(crate) layouts: Layouts,
}

impl fmt::Debug for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Workspace")
            .field("name", &self.name)
            .field("windows", &self.windows)
            .field("layout", &Option::<u32>::None)
            .finish()
    }
}

impl Workspace {
    // * PUBLIC METHODS * //

    /// Creates a new workspace.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            windows: ClientRing::new(),
            layouts: Ring::new(),
        }
    }

    /// Creates a new workspace with the given layouts.
    /// 
    /// # Panics
    /// 
    /// This function panics if one or more of the invariants
    /// on [`Layouts`] are not upheld.
    pub fn with_layouts<I>(name: &str, layouts: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn Layout>>
    {
        Self {
            name: name.into(),
            windows: ClientRing::new(),
            layouts: Layouts::with_layouts_validated(layouts)
                .expect("validation failed"),
        }
    }

    /// Creates a workspace from a given specification.
    pub fn from_spec(spec: WorkspaceSpec, available_layouts: &Layouts) -> Result<Self> {
        let mut layouts = Vec::new();
        for name in spec.layouts {
            if let Some((_, l)) = available_layouts.element_by(|l| name == l.name()) {
                layouts.push(l.boxed());
            } else {
                error!("could not find layout with name {}", name);
            }
        }

        Ok(Self {
            name: spec.name,
            windows: ClientRing::new(),
            layouts: Layouts::with_layouts_validated(layouts)?,
        })
    }

    /// Sets the layout to use and applies it to all currently mapped windows.
    pub fn set_layout<X: XConn>(
        &mut self,
        layout: &str,
        conn: &X,
        scr: &Screen,
    ) {
        let Some((idx, _)) = self.layouts.element_by(|ws| ws.name() == layout) else {
            warn!("No layout with name `{}`", layout);
            return
        };
        self.layouts.set_focused(idx);
        self.relayout(conn, scr);
    }

    /// Cycles in the given direction to the next layout, and
    /// applies it.
    pub fn cycle_layout<X: XConn>(
        &mut self, dir: Direction, conn: &X, scr: &Screen
    ) {
        self.layouts.cycle_focus(dir);
        self.relayout(conn, scr);
    }

    /// Switches to the given layout, and applies it.
    pub fn switch_layout<S: AsRef<str>, X: XConn>(
        &mut self, name: S, conn: &X, scr: &Screen
    ) {
        if let Some((idx, _)) = self.layouts.element_by(|l| l.name() == name.as_ref()) {
            self.layouts.set_focused(idx);
            self.relayout(conn, scr);
        } else {
            error!("could not find layout {}", name.as_ref());
        }
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

    /// Returns the name of the workspace.
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name
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

    #[inline]
    /// Returns an iterator over all the clients currently in the layout.
    pub fn clients_in_layout(&self) -> impl Iterator<Item = &Client> {
        self.windows.iter().filter(|w| !w.is_off_layout())
    }

    #[inline]
    /// Returns a mutable iterator over all the clients currently in the layout.
    pub fn clients_in_layout_mut(&mut self) -> impl Iterator<Item = &mut Client> {
        self.windows.iter_mut().filter(|w| !w.is_off_layout())
    }

    /// Returns the number of windows managed by the layout.
    ///
    /// Since a workspace can contain both floating and tiled windows,
    /// this returns the number of tiled windows only.
    pub fn managed_count(&self) -> usize {
        self.windows.iter().filter(|win| !win.is_off_layout()).count()
    }

    /// Returns the number of floating windows in the workspace.
    ///
    /// Since a workspace can contain both floating and tiled windows,
    /// this returns the number of floating windows only.
    pub fn floating_count(&self) -> usize {
        self.windows.iter().filter(|win| win.is_off_layout()).count()
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
    #[inline]
    pub fn is_floating(&self) -> bool {
        let Some(layout) = self.layouts.focused() else {
            warn!("No currently focused layout in workspace `{}`", self.name);
            return false
        };
        matches!(layout.style(), LayoutType::Floating)
    }

    /// Returns the name of the workspace's current layout.
    #[inline]
    pub fn layout(&self) -> &str {
        let Some(layout) = self.layouts.focused() else {
            warn!("No currently focused layout in workspace `{}`", self.name);
            return ""
        };
        layout.name()
    }

    /// Returns the style of the workspace's current layout.
    #[inline]
    pub fn layout_style(&self) -> LayoutType {
        let Some(layout) = self.layouts.focused() else {
            warn!("No currently focused layout in workspace `{}`", self.name);
            return LayoutType::Tiled
        };
        layout.style()
    }

    /// Returns the `Some(idx)` where `idx` is the index of the
    /// Client in its underlying ring, or `None` if the Client
    /// does not exist.
    #[inline]
    pub fn contains(&self, window: XWindowID) -> Option<usize> {
        self.windows.get_idx(window)
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
        let layouts = self.layouts.gen_layout(self, scr);
        self.apply_layout(conn, layouts);
    }

    /// Adds a window to the workspace in the layout.
    pub fn add_window_on_layout<X: XConn>(&mut self, conn: &X, scr: &Screen, window: XWindowID) {
        self._add_window(conn, scr, Client::new(window, conn))
    }

    /// Adds a window to the workspace off the layout.
    pub fn add_window_off_layout<X: XConn>(&mut self, conn: &X, scr: &Screen, window: XWindowID) {
        self._add_window(conn, scr, Client::outside_layout(window, conn))
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
            if win.is_off_layout() {
                Ok(Some(self._del_window(conn, scr, id, false)))
            } else {
                Ok(Some(self._del_window(conn, scr, id, true)))
            }
        } else {
            // fail silently (this accounts for spurious unmap events)
            Ok(None)
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

    /// Cycles the focus to the next window in the workspace.
    pub fn cycle_focus<X: XConn>(&mut self, conn: &X, dir: Direction) {
        // get the currently focused window's ID
        if self.windows.focused_mut().is_none() {
            error!("cycle_focus for ws {}: nothing focused", self.name);
            return;
        };
        //internally, cycle focus
        self.windows.cycle_focus(dir);

        // focus window
        self.focus_window(conn, self.focused_client().unwrap().id());
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

    /// Toggles fullscreen on the currently focused window.
    pub fn toggle_focused_fullscreen<X: XConn>(
        &mut self, conn: &X, scr: &Screen
    ) {
        todo!()
        /* 
        * 1. Actually apply the geometry
        * 2. Update EWMH properties on the server, if needed
        */
    }

    /// Toggles the state of the currently focused window between off or in layout.
    pub fn toggle_focused_state<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        // If we have a focused window
        if let Some(win) = self.windows.focused() {
            debug!("toggling state of focused window {}", win.id());
            if win.is_off_layout() {
                self.add_to_layout(conn, win.id(), scr)
            } else {
                self.remove_from_layout(conn, win.id(), scr)
            }
        }
    }

    /// Sets the focused window to be managed by the layout.
    ///
    /// Is effectively a no-op if the workspace is in a floating-style layout.
    pub fn add_to_layout<X: XConn>(&mut self, conn: &X, id: XWindowID, scr: &Screen) {
        debug!("Setting focused to tiled");

        if let Some(win) = self.windows.lookup_mut(id) {
            win.set_on_layout();
            self.relayout(conn, scr);
        }
    }

    /// Removes the focused window from being managed by the layout, effectively
    /// turning it into a floating window regardless of the current layout style.
    pub fn remove_from_layout<X: XConn>(&mut self, conn: &X, id: XWindowID, scr: &Screen) {
        debug!("Setting focused to floating");
        if let Some(win) = self.windows.lookup_mut(id) {
            win.set_off_layout();
            self.relayout(conn, scr);
        }
    }

    /// Sends an update to the currently focused layout, and applies
    /// and changes that may have taken place.
    pub fn update_focused_layout<U, X: XConn>(&mut self, conn: &X, scr: &Screen, msg: U)
    where
        U: IntoUpdate
    {
        self.layouts.send_update(msg.into_update());
        self.relayout(conn, scr);
    }

    // * PRIVATE METHODS * //

    #[instrument(level = "debug", skip(self, conn, scr, window))]
    fn _add_window<X: XConn>(&mut self, conn: &X, scr: &Screen, mut window: Client) {
        trace!("adding window {:#?}", window);
        // Set supported protocols
        window.set_supported(conn);
        // Configure window with a border width
        window.configure(conn, &[ClientConfig::BorderWidth(BORDER_WIDTH)]);

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

    /// Deletes a window
    fn _del_window<X: XConn>(&mut self,
        conn: &X, 
        scr: &Screen, 
        id: XWindowID,
        on_layout: bool,
    ) -> Client {
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

        if on_layout {
            self.relayout(conn, scr);
        }

        window
    }

    /// Pushes a window directly without calling the layout.
    pub(crate) fn put_window(&mut self, window: Client) {
        if window.is_off_layout() {
            self.windows.push(window);
        } else {
            self.windows.append(window);
        }
    }

    /// Takes a window directly without calling the layout.
    pub(crate) fn take_window<X: XConn>(&mut self, window: XWindowID, conn: &X) -> Option<Client> {
        let mut window = self.windows.remove_by_id(window)?;
        window.unmap(conn);
        Some(window)
    }

    fn apply_layout<X: XConn>(&mut self, conn: &X, layouts: Vec<LayoutAction>) {
        // get all off_layout windows and stack them above all tiled
        for floater in self.clients_mut().filter(|c| c.is_off_layout()) {
            floater.configure(conn, &[ClientConfig::StackingMode(StackMode::Above)])
        }

        for rsaction in layouts {
            match rsaction {
                LayoutAction::Resize {id, geom } => {

                    let window = self.windows.lookup_mut(id).unwrap();
                    window.set_and_update_geometry(conn, geom);
                }
                LayoutAction::Map(_) => {
                    //todo
                }
                LayoutAction::Unmap(_) => {
                    //todo
                }
            }
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
}
