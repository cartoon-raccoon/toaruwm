//! This module exports `Desktop` and `Screen`.
//! 
//! `Desktop` is the main type handled directly by `WindowManager`.
//! It encapsulates multiple workspaces, and handles sending
//! windows between workspaces.
//! 
//! `Screen` represents a physical monitor that X is connected to.
//! It encapsulates monitor resolution and is used by the tiling
//! algorithms to resize windows.

use crate::x::{XConn, XWindowID};
use crate::types::{Ring, Geometry, Direction, Selector};
use crate::core::{Workspace, Client};
use crate::layouts::{LayoutType, LayoutFn};

/// Represents a physical monitor.
#[derive(Clone, Copy, Debug)]
pub struct Screen {
    pub(crate) root_id: XWindowID,
    pub(crate) effective_geom: Geometry,
    pub(crate) true_geom: Geometry,
    pub(crate) idx: i32,
}

impl Screen {
    pub fn new(
        screen_idx: i32, 
        geom: Geometry, 
        root_id: XWindowID
    ) -> Self {
        Self {
            root_id,
            effective_geom: geom,
            true_geom: geom,
            idx: screen_idx,
        }
    }

    pub fn update_effective(&mut self,) {

    }

    pub fn true_geom(&self) -> Geometry {
        self.true_geom
    }

    pub fn effective_geom(&self) -> Geometry {
        self.effective_geom
    }
}


#[derive(Debug, Clone)]
pub struct Desktop {
    pub(crate) workspaces: Ring<Workspace>,
    current: usize,
}

impl Desktop {
    pub fn new(layout: LayoutType, lfn: Option<LayoutFn>, wksps: Vec<String>) -> Self {
        Self {
            workspaces: {
                let mut workspaces = Ring::with_capacity(wksps.len());

                for name in wksps {
                    workspaces.push(Workspace::with_layout(layout.clone(), lfn, &name));
                }

                workspaces.set_focused(0);
                workspaces
            },
            current: 0,
        }
    }

    /// The layout of the current workspace.
    pub fn current_layout(&self) -> &LayoutType {
        self.current().layout()
    }

    pub fn current_client(&self) -> Option<&Client> {
        match self.workspaces.focused() {
            Some(ws) => ws.focused_client(),
            None => None
        }
    }

    pub fn current_client_mut(&mut self) -> Option<&mut Client> {
        match self.workspaces.focused_mut() {
            Some(ws) => ws.focused_client_mut(),
            None => None
        }
    }

    /// Test whether a certain window is already managed.
    pub fn is_managing(&self, id: XWindowID) -> bool {
        self.workspaces.iter().any(|ws| ws.contains_window(id))
    }

    /// Returns a reference to the current workspace.
    pub fn current(&self) -> &Workspace {
        &self.workspaces[self.current]
    }

    /// Returns a mutable reference to the current workspace.
    pub fn current_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.current]
    }

    pub fn current_idx(&self) -> usize {
        self.workspaces.focused.unwrap()
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
        self.goto(&name, conn, scr);
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
    pub fn goto<X: XConn>(&mut self, name: &str, conn: &X, scr: &Screen) {
        let new_idx = self.workspaces.index(Selector::Condition(&|ws| ws.name == name));
        if new_idx.is_none() {
            error!("No workspace {} found", name);
            return
        }
        let new_idx = new_idx.unwrap();
        if self.current == new_idx {
            return
        }
        debug!("Goto desktop {}", new_idx);

        self.current_mut().deactivate(conn);
        
        self.current = new_idx;

        if let Some(ws) = self.get_mut(self.current) {
            ws.activate(conn, scr);
        } else {
            error!("No workspace found for index {}", new_idx);
        }
    }

    /// Send a window to a given workspace.
    pub fn send_window_to<X: XConn>(&mut self, name: &str, conn: &X, scr: &Screen) {
        debug!("Attempting to send window to workspace {}", name);
        if let Some(window) = self.current_mut().take_focused_window(conn, scr) {
            debug!("Sending window {} to workspace {}", window.id(), name);
            if let Some(ws) = self.find_mut(name) {
                ws.push_window(window);
            } else {
                error!("Cannot find workspace named {}", name);
            }
        } else {
            info!("No focused window for workspace {}", name);
        }
        self.current_mut().relayout(conn, scr);
    }
}