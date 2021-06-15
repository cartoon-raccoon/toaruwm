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
    ResizeAction,
    LayoutFn,
};
use crate::x::{XConn, XWindowID, core::StackMode};

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

#[allow(unused_variables)]
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

    pub fn focused_client(&self) -> Option<&Client> {
        self.windows.focused()
    }

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

    pub fn add_window_floating<X: XConn>(&mut self, 
        conn: &X, scr: &Screen, id: XWindowID
    ) {
        fn_ends!("add_window_floating");

        let mut window = Client::floating(id, conn);

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
            if ptr.child == conn.get_root().id || ptr.child == id {
                self.focus_window(conn, id);
            } else if let Some(focused) = self.windows.focused_mut() {
                focused.set_border(conn, BorderStyle::Unfocused);
            } else {
                window.set_border(conn, BorderStyle::Unfocused);
            }
        }

        window.change_attributes(conn, &[ClientAttrs::EnableClientEvents]);

        self.windows.push(window);
        self.relayout(conn, scr);
    }

    pub fn add_window_tiled<X: XConn>(&mut self, 
        conn: &X, scr: &Screen, id: XWindowID
    ) {
        todo!("tiling algorithm not implemented")
    }

    #[allow(mutable_borrow_reservation_conflict)]
    fn del_window_floating<X: XConn>(&mut self, 
        conn: &X, scr: &Screen, id: XWindowID
    ) -> Client {
        let mut window = self.windows.remove_by_id(id)
        //todo: return Result instead
        .expect("Could not find window");

        window.change_attributes(conn, &[ClientAttrs::DisableClientEvents]);
        window.unmap(conn);

        if let Some(idx) = self.windows.get_idx(id) {
            if idx == 0 {
                if let Some(next) = self.windows.get(0) {
                    window_stack_and_focus(self, conn, next.id())
                }
            }
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
        todo!("tiling algorithm not implemented")
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

    pub fn relayout<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        let layouts = self.layoutter.gen_layout(&self, scr);
        self.apply_layout(conn, layouts);
    }

    fn apply_layout<X: XConn>(
        &mut self, 
        conn: &X, 
        layouts: Vec<ResizeAction>
    ) {
        for rsaction in layouts {
            let window = self.windows.lookup_mut(rsaction.id()).unwrap();
            window.set_and_update_geometry(conn, rsaction.geometry());
        }
    }

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
        }
    }

    pub fn unfocus_window<X: XConn>(&mut self, conn: &X, window: XWindowID) {
        // remove focus if window to unfocus is currently focused
        if let Some(win) = self.windows.focused() {
            if win.id() == window {
                self.windows.unset_focused();
            }
        }
        conn.change_window_attributes(window, &[
            ClientAttrs::BorderColour(BorderStyle::Unfocused)
        ]).unwrap_or_else(|e| error!("{}", e));
    }

    pub fn cycle_focus<X: XConn>(&mut self, conn: &X, dir: Direction) {
        use BorderStyle::*;

        //change currently focused border colour to unfocused
        if let Some(win) = self.windows.focused_mut() {
            win.set_border(conn, Unfocused);
        }
        
        //internally, cycle focus
        self.windows.cycle_focus(dir);

        // change focus colours
        if self.windows.focused().is_some() {
            let focused = self.windows.focused().unwrap().id();

            window_stack_and_focus(self, conn, focused);
        }
    }

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

    pub fn toggle_focused_state<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        debug!("Toggling state of focused window {:#?}", self.windows.focused());
        let master = self.master;
        // If we have a focused window
        if let Some(win) = self.windows.focused_mut() {
            // set a stack variable to avoid overlapping lifetimes
            let win_id = win.id();
            if win.is_floating() { //toggling to tiled
                debug!("Toggling window state");
                win.toggle_state();
                // if we have no master
                if master.is_none() {
                    debug!("No master, setting master");
                    self.set_master(win_id);
                }
                // keep floating windows on top
            } else { //toggling to floating
                debug!("Toggling window state");

                // toggle state and stack above
                win.toggle_state();
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

    pub fn set_master(&mut self, master_id: XWindowID) {
        if !self.windows.contains(master_id) {
            error!("set_master: No such window {}", master_id);
            return
        }
        self.master = Some(master_id);
        let idx = self.windows.get_idx(master_id).unwrap();
        self.windows.move_front(idx);
    }

    pub fn unset_master(&mut self) {
        if self.tiled_count() > 0 {
            error!("unset_master: Workspace still has tiled windows");
        }
        self.master = None;
    }

    #[inline(always)]
    pub fn is_master(&mut self, id: XWindowID) -> bool {
        if let Some(win) = self.master {
            return win == id
        }
        false
    }

    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline(always)]
    pub fn master(&self) -> Option<XWindowID> {
        self.master
    }

    pub fn tiled_count(&self) -> usize {
        self.windows.iter().filter(|win| win.is_tiled()).count()
    }

    pub fn floating_count(&self) -> usize {
        self.windows.iter().filter(|win| win.is_floating()).count()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    #[inline(always)]
    pub fn is_tiling(&self) -> bool {
        !self.is_floating()
    }
    
    #[inline(always)]
    pub fn is_floating(&self) -> bool {
        matches!(self.layoutter.layout(), LayoutType::Floating)
    }

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