//! Types used for desktop management.
//!
//! This module exports `Desktop` and `Screen`.
//!
//! `Desktop` is the main type handled directly by `WindowManager`.
//! It encapsulates multiple workspaces, and handles sending
//! windows between workspaces.
//!
//! `Screen` represents a physical monitor that X is connected to.
//! It encapsulates monitor resolution and is used by the tiling
//! algorithms to resize windows.
#![allow(dead_code)]

use tracing::debug;

use crate::core::{Window, Workspace};
use crate::layouts::{Layout, Layouts};
use crate::config::RuntimeConfig;
use crate::platform::{Platform, PlatformOutput};
use crate::types::{
    Cardinal, Direction, Rectangle, Logical, Ring, Selector
};
use crate::{Result, ToaruError::*};

use super::WorkspaceSpec;

/// Represents a physical monitor.
#[derive(Debug, Clone)]
pub struct Screen<P: Platform> {
    pub(crate) name: String,
    pub(crate) output: P::Output,
    /// The usable geometry of the Screen.
    pub(crate) effective_geom: Rectangle<i32, Logical>,
    /// The index of the Screen.
    pub(crate) idx: i32,
}

impl<P: Platform> Screen<P> {

    /// Creates a new Screen with the given output.
    pub fn new(output: P::Output, screen_idx: i32) -> Self {
        let effective_geom = output.geometry().unwrap_or_else(|| Rectangle::zeroed());
        Self {
            name: output.name(),
            output,
            effective_geom,
            idx: screen_idx,
        }
    }

    /// Updates the effective area of the screen by trimming off
    /// a section in the given direction.
    pub fn update_effective(&mut self, dir: Cardinal, trim: i32) {
        self.effective_geom = self.effective_geom.trim(trim, dir);
    }

    /// Sets the effective geometry of the screen.
    pub fn set_effective(&mut self, geom: Rectangle<i32, Logical>) {
        self.effective_geom = geom;
    }

    /// Returns the true geometry of the Screen.
    pub fn true_geom(&self) -> Rectangle<i32, Logical> {
        self.output.geometry().unwrap_or_else(|| Rectangle::zeroed())
    }

    /// Returns the effective Geometry of the Screen.
    pub fn effective_geom(&self) -> Rectangle<i32, Logical> {
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
#[derive(Debug)]
pub struct Desktop<P: Platform> {
    // * focused should never be none
    pub(crate) workspaces: Ring<Workspace<P>>,
    last_ws: usize,
}

impl<P: Platform> Desktop<P> {
    /// Creates a new `Desktop`.
    pub fn new<N, R, L>(wksps: N, layouts: L) -> Result<Self, P>
    where
        N: IntoIterator<IntoIter = R>,
        R: DoubleEndedIterator<Item = WorkspaceSpec>,
        L: IntoIterator<Item = Box<dyn Layout<P>>>,
        P: Platform,
    {
        let mut desktop = Self {
            workspaces: {
                let mut workspaces = Ring::new();

                let ins = Layouts::with_layouts_validated(layouts)?;
                for spec in wksps.into_iter().rev() {
                    workspaces.push(Workspace::from_spec(spec, &ins)?);
                }

                workspaces.set_focused(0);
                workspaces
            },
            last_ws: 0,
        };

        desktop.workspaces.set_focused(0);

        Ok(desktop)
    }

    //* Retrieval and Convenience Methods *//

    /// The layout of the current workspace.
    pub fn current_layout(&self) -> &str {
        self.current().layout()
    }

    /// Test whether a certain window is already managed.
    pub fn is_managing(&self, id: P::WindowId) -> bool {
        self.workspaces.iter().any(|ws| ws.contains_window(id))
    }

    /// Get a reference to the focused client of the focused workspace.
    pub fn current_client(&self) -> Option<&Window<P>> {
        match self.workspaces.focused() {
            Some(ws) => ws.focused_client(),
            None => None,
        }
    }

    /// Get a mutable reference to the focused client of the focused
    /// workspace.
    pub fn current_client_mut(&mut self) -> Option<&mut Window<P>> {
        match self.workspaces.focused_mut() {
            Some(ws) => ws.focused_client_mut(),
            None => None,
        }
    }

    /// Returns a reference to the current workspace.
    pub fn current(&self) -> &Workspace<P> {
        &self.workspaces[self.current_idx()]
    }

    /// Returns a mutable reference to the current workspace.
    pub fn current_mut(&mut self) -> &mut Workspace<P> {
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
    pub fn retrieve(&mut self, window: P::WindowId) -> Option<(&Workspace<P>, usize)> {
        for ws in self.workspaces.iter() {
            if let Some(idx) = ws.contains(window) {
                return Some((ws, idx));
            }
        }

        None
    }

    /// `retrieve`'s mutable version.
    pub fn retrieve_mut(&mut self, window: P::WindowId) -> Option<(&mut Workspace<P>, usize)> {
        for ws in self.workspaces.iter_mut() {
            if let Some(idx) = ws.contains(window) {
                return Some((ws, idx));
            }
        }

        None
    }

    /// Get a reference to a workspace by its index
    pub fn get(&self, idx: usize) -> Option<&Workspace<P>> {
        if idx + 1 >= self.workspaces.len() {
            return None;
        }

        Some(&self.workspaces[idx])
    }

    /// Get a mutable reference to a workspace by index.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Workspace<P>> {
        if idx + 1 > self.workspaces.len() {
            return None;
        }

        Some(&mut self.workspaces[idx])
    }

    /// Find a workspace by its name.
    ///
    /// Returns an immutable reference.
    pub fn find(&self, name: &str) -> Option<&Workspace<P>> {
        self.workspaces
            .element_by(|ws| ws.name == name)
            .map(|(_, ws)| ws)
    }

    /// Find a workspace by its name.
    ///
    /// Returns a mutable reference.
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Workspace<P>> {
        self.workspaces
            .element_by_mut(|ws| ws.name == name)
            .map(|(_, ws)| ws)
    }

    //* Mutator and Manipulation Methods *//

    /// Cycle workspaces in given direction.
    pub fn cycle_to<C>(
        &mut self,
        scr: &Screen<P>,
        cfg: &C,
        direction: Direction,
    ) -> Result<(), P>
    where
        C: RuntimeConfig,
    {
        debug!("Cycling workspaces in direction {:?}", direction);
        self.workspaces.cycle_focus(direction);

        // i hate you, borrow checker
        let name: String;
        if let Some(ws) = self.workspaces.focused() {
            name = ws.name().into();
        } else {
            return Err(OtherError("Focused should be Some".into()));
        }
        self.go_to(&name, scr, cfg)
    }

    /// Switch to a given workspace by its name.
    pub fn go_to<C>(&mut self, name: &str, scr: &Screen<P>, cfg: &C) -> Result<(), P>
    where
        C: RuntimeConfig,
    {
        debug!("Going to workspace with name '{}'", name);

        let new_idx = self
            .workspaces
            .index(Selector::Condition(&|ws| ws.name == name));
        if new_idx.is_none() {
            return Err(UnknownWorkspace(name.into()));
        }
        let new_idx = new_idx.unwrap();
        if self.current_idx() == new_idx {
            //todo: go to last workspace if same
            return Ok(());
        }

        self.current_mut().deactivate();
        self.set_current(new_idx);

        debug!("Goto workspace idx {}", new_idx);

        if let Some(ws) = self.get_mut(self.current_idx()) {
            ws.activate(scr, cfg);
        } else {
            return Err(UnknownWorkspace(name.into()));
        }

        Ok(())
    }
    /// Sends the currently focused window to the specified workspace.
    pub fn send_focused_to<C>(&mut self,name: &str, scr: &Screen<P>, cfg: &C) -> Result<(), P>
    where
        C: RuntimeConfig,
    {
        debug!("Attempting to send window to workspace {}", name);
        let winid = if let Some(window) = self.current().focused_client() {
            window.id()
        } else {
            debug!("No focused window in workspace {}", name);
            return Ok(());
        };
        self.send_window_to(winid, name, scr, cfg)
    }

    /// Send a window to a given workspace.
    pub fn send_window_to<C>(&mut self, id: P::WindowId, name: &str, scr: &Screen<P>, cfg: &C) -> Result<(), P>
    where
        C: RuntimeConfig,
    {
        debug!("Attempting to send window to workspace {}", name);
        let Some(window) = self.current_mut().take_window(id) else {
            return Err(UnknownClient(id.clone()))
        };
        debug!("Sending window {:?} to workspace {}", window.id(), name);
        let Some(ws) = self.find_mut(name) else {
            // if workspace was not found, put it back
            self.current_mut().put_window(window);
            return Err(UnknownWorkspace(name.into()))
        };
        ws.put_window(window);
        if ws.focused_client().is_none() {
            ws.windows.set_focused_by_winid(id);
        }
        self.current_mut().relayout(scr, cfg);
        Ok(())
    }
}
