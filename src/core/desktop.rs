//! This module exports `Desktop` and `Screen`.
//! 
//! `Desktop` is the main type handled directly by `WindowManager`.
//! It encapsulates multiple workspaces, and handles sending
//! windows between workspaces.
//! 
//! `Screen` represents a physical monitor that X is connected to.
//! It encapsulates monitor resolution and is used by the tiling
//! algorithms to resize windows.
//! 

use tracing::{debug};

use crate::x::{XConn, XWindowID, Atom, Property};
use crate::types::{
    Ring, Geometry, 
    Direction, Selector,
    Cardinal,
};
use crate::core::{Workspace, Client};
use crate::layouts::{LayoutType, LayoutFn};
use crate::{Result, ToaruError::*};

/// Represents a physical monitor.
#[derive(Clone, Debug)]
pub struct Screen {
    pub(crate) root_id: XWindowID,
    pub(crate) effective_geom: Geometry,
    pub(crate) true_geom: Geometry,
    pub(crate) idx: i32,
    pub(crate) wix: Vec<String>,
}

impl Screen {
    pub fn new(
        screen_idx: i32, 
        geom: Geometry, 
        root_id: XWindowID,
        wix: Vec<String>,
    ) -> Self {
        Self {
            root_id,
            effective_geom: geom,
            true_geom: geom,
            idx: screen_idx,
            wix,
        }
    }

    pub fn add_workspace<S: Into<String>>(&mut self, wsname: S) {
        self.wix.push(wsname.into());
    }

    pub fn update_effective(&mut self, dir: Cardinal, trim: i32) {
        self.true_geom = self.true_geom.trim(trim, dir);
    }

    pub fn true_geom(&self) -> Geometry {
        self.true_geom
    }

    pub fn effective_geom(&self) -> Geometry {
        self.effective_geom
    }
}

/// Encapsulates all the workspaces managed by the window manager.
/* 
* technically I could just set this as a field in the windowmanager,
* but I wanted to encapsulate this away so that the window manager's
* delegated tasks are more specific to the general operation of the
* manager itself, and the workspace handling logic can be implemented
* separately.
*/
#[derive(Debug, Clone)]
pub struct Desktop {
    // * focused should never be none
    pub(crate) workspaces: Ring<Workspace>,
    last_ws: usize,
}

impl Desktop {
    pub fn new(layout: LayoutType, lfn: Option<LayoutFn>, wksps: Vec<String>) -> Self {
        let mut desktop = Self {
            workspaces: {
                let mut workspaces = Ring::with_capacity(wksps.len());

                for name in wksps.into_iter().rev() {
                    workspaces.push(Workspace::with_layout(layout.clone(), lfn, &name));
                }

                workspaces.set_focused(0);
                workspaces
            },
            last_ws: 0,
        };

        desktop.workspaces.set_focused(0);

        desktop
    }

    //* Retrieval and Convenience Methods *//

    /// The layout of the current workspace.
    pub fn current_layout(&self) -> &LayoutType {
        self.current().layout()
    }

    /// Test whether a certain window is already managed.
    pub fn is_managing(&self, id: XWindowID) -> bool {
        self.workspaces.iter().any(|ws| ws.contains_window(id))
    }

    /// Get a reference to the focused client of the focused workspace.
    pub fn current_client(&self) -> Option<&Client> {
        match self.workspaces.focused() {
            Some(ws) => ws.focused_client(),
            None => None
        }
    }

    /// Get a mutable reference to the focused client of the focused
    /// workspace.
    pub fn current_client_mut(&mut self) -> Option<&mut Client> {
        match self.workspaces.focused_mut() {
            Some(ws) => ws.focused_client_mut(),
            None => None
        }
    }

    /// Returns a reference to the current workspace.
    pub fn current(&self) -> &Workspace {
        &self.workspaces[self.current_idx()]
    }

    /// Returns a mutable reference to the current workspace.
    pub fn current_mut(&mut self) -> &mut Workspace {
        let current = self.current_idx();
        &mut self.workspaces[current]
    }

    /// Name of the workspace in focus.
    pub fn current_name(&self) -> &str {
        &self.current().name
    }

    pub(crate) fn current_idx(&self) -> usize {
        self.workspaces.focused.expect("Focused index not set")
    }

    pub(crate) fn set_current(&mut self, idx: usize) {
        self.workspaces.set_focused(idx);
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

    //* Mutator and Manipulation Methods *//
    
    /// Cycle workspaces in given direction.
    pub fn cycle_workspace<X: XConn>(&mut self, 
        conn: &X, 
        scr: &Screen, 
        direction: Direction
    ) -> Result<()> {
        debug!("Cycling workspaces in direction {:?}", direction);
        self.workspaces.cycle_focus(direction);

        // i hate you, borrow checker
        let name: String;
        if let Some(ws) = self.workspaces.focused() {
            name = ws.name().into();
        } else {
            return Err(OtherError("Focused should be Some".into()))
        }
        self.goto(&name, conn, scr)
    }

    /// Switch to a given workspace by its name.
    pub fn goto<X: XConn>(&mut self, name: &str, conn: &X, scr: &Screen) -> Result<()> {
        debug!("Going to workspace with name '{}'", name);

        let new_idx = self.workspaces.index(Selector::Condition(&|ws| ws.name == name));
        if new_idx.is_none() {
            return Err(UnknownWorkspace(name.into()))
        }
        let new_idx = new_idx.unwrap();
        if self.current_idx() == new_idx {
            //todo: go to last workspace if same
            return Ok(())
        }

        conn.set_property(
            conn.get_root().id,
            Atom::NetCurrentDesktop.as_ref(),
            Property::Cardinal(new_idx as u32)
        )?;
        
        self.current_mut().deactivate(conn);
        self.set_current(new_idx);
        
        debug!("Goto workspace idx {}", new_idx);

        if let Some(ws) = self.get_mut(self.current_idx()) {
            ws.activate(conn, scr);
        } else {
            return Err(UnknownWorkspace(name.into()))
        }
        
        Ok(())
    }

    pub fn send_focused_to<X: XConn>(&mut self, name: &str, conn: &X, scr: &Screen) -> Result<()> {
        debug!("Attempting to send window to workspace {}", name);
        let winid = if let Some(window) = self.current().focused_client() {
            window.id()
        } else {
            debug!("No focused window in workspace {}", name);
            return Ok(())
        };
        self.send_window_to(winid, name, conn, scr)
    }

    /// Send a window to a given workspace.
    pub fn send_window_to<X: XConn>(&mut self, id: XWindowID, name: &str, conn: &X, scr: &Screen) -> Result<()> {
        debug!("Attempting to send window to workspace {}", name);
        let window = self.current_mut().del_window(conn, scr, id)?;
        debug!("Sending window {} to workspace {}", window.id(), name);
        if let Some(ws) = self.find_mut(name) {
            ws.push_window(window);
        } else {
            return Err(UnknownWorkspace(name.into()))
        }
        self.current_mut().relayout(conn, scr);
        Ok(())
    }
}