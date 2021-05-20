use crate::core::{
    window::{Client, ClientRing},
    desktop::Screen,
};
use crate::types::{
    BorderStyle, Direction
};
use crate::layouts::{
    LayoutType, 
    LayoutEngine, 
    ResizeAction
};
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
        self.relayout(conn, scr);
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
        debug!("Current master is {:?}", self.master);
        debug!("{:#?}", &self.windows);
        todo!()
    }

    pub fn del_window<X: XConn>(&mut self, conn: &X, scr: &Screen, id: XWindowID) -> Client {
        todo!()
    }

    /// Pushes a window directly.
    pub(crate) fn push_window(&mut self, window: Client) {
        function_ends!("[start] workspace::push_window");
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
        function_ends!("[end] workspace::push_window");
    }

    pub fn relayout<X: XConn>(&mut self, conn: &X, scr: &Screen) {
        let layouts = self.layoutter.gen_layout(&self, scr);
        self.apply_layout(conn, scr, layouts);
    }

    fn apply_layout<X: XConn>(
        &mut self, 
        conn: &X, 
        scr: &Screen, 
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
    pub fn layout(&self) -> LayoutType {
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
    conn.change_window_attributes(window, &util::disable_events());

    let win = ws.windows.lookup_mut(window).unwrap();

    // if there is a focused window, stack it above
    // if let Some(win) = ws.windows.focused() {
    //     debug!("Focusing window {}", win.id());
    //     conn.configure_window(window, &utils::stack_above(win.id()));
    // }

    
    // focus to current window
    win.set_border(conn, Focused);
    conn.set_input_focus(window);

    // re-enable events
    conn.change_window_attributes(window, &util::child_events());
}