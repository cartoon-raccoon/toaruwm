use crate::x::{XConn, XWindow, XWindowID};
use crate::types::{Ring, Geometry, Direction, Selector};
use crate::core::Workspace;
use crate::layouts::LayoutType;

/// Represents a physical monitor.
#[derive(Clone, Copy, Debug)]
pub struct Screen {
    pub xwindow: XWindow,
    pub idx: i32,
}

const MAX_WKSPACES: usize = 10;

impl Screen {
    pub fn new(screen_idx: i32, root_id: XWindowID) -> Self {
        Self {
            xwindow: XWindow::from(root_id),
            idx: screen_idx,
        }
    }

    pub fn geom(&self) -> Geometry {
        self.xwindow.geom
    }
}


#[derive(Clone)]
pub struct Desktop {
    pub(crate) workspaces: Ring<Workspace>,
    current: usize,
}

impl Desktop {
    pub fn new(layout: LayoutType) -> Self {
        Self {
            workspaces: {
                let mut workspaces = Ring::with_capacity(MAX_WKSPACES);

                for i in 0..MAX_WKSPACES {
                    workspaces.push(Workspace::with_layout(layout, &i.to_string()));
                }

                workspaces.set_focused(0);
                workspaces
            },
            current: 0,
        }
    }

    /// Returns a reference to the current workspace.
    pub fn current(&self) -> &Workspace {
        &self.workspaces[self.current]
    }

    /// Returns a mutable reference to the current workspace.
    pub fn current_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.current]
    }

    /// Get a reference to the workspace containing the window
    /// and the window's index in the workspace.
    pub fn retrieve(&mut self, window: XWindowID) -> Option<(&Workspace, usize)> {
        for ws in self.workspaces.iter() {
            if let Some(idx) = ws.contains(window) {
                return Some((ws, idx))
            }
        }

        None
    }

    /// `retrieve`'s mutable version.
    pub fn retrieve_mut(&mut self, window: XWindowID) -> Option<(&mut Workspace, usize)> {
        for ws in self.workspaces.iter_mut() {
            if let Some(idx) = ws.contains(window) {
                return Some((ws, idx))
            }
        }

        None
    }

    pub fn cycle_workspace<X: XConn>(&mut self, 
        conn: &X, 
        scr: &Screen, 
        direction: Direction
    ) {
        debug!("Cycling workspaces in direction {:?}", direction);
        self.workspaces.cycle_focus(direction);

        // i hate you, borrow checker
        let name: String;
        if let Some(ws) = self.workspaces.focused() {
            name = ws.name().into();
        } else {
            error!("Focused should be Some");
            return
        }
        self.goto(conn, scr, &name);
    }

    /// Get a reference to a workspace by its index
    pub fn get(&self, idx: usize) -> Option<&Workspace> {
        if idx + 1 >= self.workspaces.len() {
            return None
        }

        Some(&self.workspaces[idx])
    }

    /// Get a mutable reference to a workspace by index.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Workspace> {
        if idx + 1 > self.workspaces.len() {
            return None
        }

        Some(&mut self.workspaces[idx])
    }

    /// Find a workspace by its name.
    /// 
    /// Returns an immutable reference.
    pub fn find(&self, name: &str) -> Option<&Workspace> {
        self.workspaces.element_by(|ws| ws.name == name).map(|(_, ws)| ws)
    }

    /// Find a workspace by its name.
    /// 
    /// Returns a mutable reference.
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Workspace> {
        self.workspaces.element_by_mut(|ws| ws.name == name).map(|(_, ws)| ws)
    }

    /// Switch to a given workspace by its name.
    pub fn goto<X: XConn>(&mut self, conn: &X, scr: &Screen, name: &str) {
        let new_idx = self.workspaces.index(Selector::Condition(&|ws| ws.name == name));
        if new_idx.is_none() {
            error!("No workspace {} found", name);
        }
        let new_idx = new_idx.unwrap();
        if self.current == new_idx {
            return
        }
        debug!("Goto desktop {}", new_idx);

        self.workspaces.get_mut(self.current).unwrap().deactivate(conn);
        
        self.current = new_idx;

        if let Some(ws) = self.get_mut(self.current) {
            ws.activate(conn, scr);
        } else {
            error!("No workspace found for index {}", new_idx);
        }
    }

    /// Send a window to a given workspace.
    pub fn send_window_to<X: XConn>(&mut self, conn: &X, scr: &Screen, name: &str) {
        debug!("Attempting to send window to workspace {}", name);
        if let Some(window) = self.current_mut().take_focused_window(conn, scr) {
            debug!("Sending window {} to workspace {}", window.id(), name);
            if let Some(ws) = self.find_mut(name) {
                ws.push_window(window);
            } else {
                error!("Cannot find workspace named {}", name);
            }
        } else {
            debug!("No focused window for workspace {}", name);
        }
        self.current_mut().relayout(conn, scr);
    }
}