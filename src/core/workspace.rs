use crate::core::{
    window::{Client, ClientRing},
    desktop::Screen,
};
use crate::layouts::{LayoutType, LayoutEngine};
use crate::x::{XConn, XWindowID};
use crate::util;

#[derive(Clone)]
pub struct Workspace {
    pub(crate) windows: ClientRing,
    pub(crate) master: Option<XWindowID>,
    pub(crate) layoutter: LayoutEngine,
}

#[allow(unused_variables)]
impl Workspace {
    /// Creates a new workspace with a specific layout.
    pub fn with_layout(layout: LayoutType) -> Self {
        Self {
            windows: ClientRing::new(),
            master: None,
            layoutter: LayoutEngine::with_layout(layout),
        }
    }

    /// Sets the layout to use and applies it to all currently mapped windows.
    pub fn set_layout<X: XConn>(&mut self, layout: LayoutType, conn: &X, scr: &Screen) {
        self.layoutter.set_layout(layout);
        self.apply_layout(conn, scr);
    }

    /// Maps all the windows in the workspace.
    pub fn activate<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        if self.windows.is_empty() {
            return
        }

        // focus the main window in the workspace
        // if floating, focus the first window
        // else (should be tiled), focus the master window
        if let LayoutType::Floating = self.layout() {
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
            window.change_attributes(conn, &util::disable_events());
            // update window geometry in the x server
            window.update_geometry(conn);
            // map window
            conn.map_window(window.id());
            // re-enable events
            window.change_attributes(conn, &util::child_events());
        }
    }

    /// Unmaps all the windows in the workspace.
    pub fn deactivate<X: XConn>(&mut self, conn: &X) {
        for window in self.windows.iter() {
            conn.change_window_attributes(window.id(), &util::disable_events());
    
            conn.unmap_window(window.id());
    
            conn.change_window_attributes(window.id(), &util::child_events());
        }
    }

    /// Adds a new window and maps it.
    pub fn add_window<X: XConn>(&mut self, conn: &X, scr: &Screen, id: XWindowID) {

    }

    pub fn del_window<X: XConn>(&mut self, conn: &X, scr: &Screen, id: XWindowID) -> Client {
        todo!()
    }

    /// Pushes a window directly.
    pub(crate) fn push_window(&mut self, window: Client) {
        
    }

    pub fn apply_layout<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        todo!()
    }

    pub fn focus_window<X: XConn>(&mut self, conn: &X, window: XWindowID) {
        
    }

    pub fn take_focused_window<X: XConn>(&mut self,conn: &X,
        screen: &Screen,
    ) -> Option<Client> {
        if let Some(window) = self.windows.focused() {
            let idx = self.windows.get_idx(window.id()).unwrap();
            let window = window.to_owned();
            self.del_window(conn, screen, window.id());

            Some(window)
        } else {
            None
        }
    }

    pub fn set_master(&mut self, id: XWindowID) {

    }

    pub fn unset_master(&mut self, id: XWindowID) {

    }

    #[inline(always)]
    pub fn is_master(&mut self, id: XWindowID) -> bool {
        if let Some(win) = self.master {
            return win == id
        }
        false
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
        if let LayoutType::Floating = self.layoutter.layout() {
            return false
        }
        true
    }

    #[inline]
    pub fn layout(&self) -> LayoutType {
        self.layoutter.layout()
    }

    #[inline]
    pub fn contains(&self, window: XWindowID) -> Option<usize> {
        self.windows.get_idx(window)
    }
}