//! Types used for desktop management.
//!
//! This module exports [`Desktop`] and [`Monitor`], as well as [`WorkspaceMux`].
//!
//! `Desktop` is the main type handled directly by `WindowManager`.
//! It encapsulates multiple workspaces, and handles sending
//! windows between workspaces. Additionally, [`WorkspaceMux`] handles multiplexing
//! various monitors between a global namespace of Workspaces.
//!
//! [`Monitor`] represents a physical monitor that is connected to your computer.
//! It encapsulates monitor resolution and is used by the tiling
//! algorithms to resize windows.
#![allow(dead_code)]

use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

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
pub struct Monitor<P: Platform> {
    pub(crate) name: String,
    pub(crate) output: P::Output,
    /// The usable geometry of the Screen.
    pub(crate) effective_geom: Rectangle<i32, Logical>,
    /// The index of the Screen.
    pub(crate) idx: i32,
}

impl<P: Platform> Monitor<P> {

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
    pub(crate) workspaces: Vec<Workspace<P>>,
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
                let mut workspaces = Vec::new();

                let ins = Layouts::with_layouts_validated(layouts)?;
                for spec in wksps.into_iter().rev() {
                    workspaces.push(Workspace::from_spec(spec, &ins)?);
                }
                workspaces
            },
            last_ws: 0,
        };

        Ok(desktop)
    }

    //* Retrieval and Convenience Methods *//

    /// Test whether a certain window is already managed.
    pub fn is_managing(&self, id: P::WindowId) -> bool {
        self.workspaces.iter().any(|ws| ws.contains_window(id))
    }

    /// Returns an iterator over the active workspaces in the desktop.
    pub fn active_workspaces(&self) -> impl Iterator<Item = &Workspace<P>> {
        self.workspaces.iter().filter(|ws| ws.is_active())
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
    pub fn find_ws(&self, name: &str) -> Option<&Workspace<P>> {
        self.workspaces.iter()
            .find(|ws| ws.name == name)
    }

    /// Find a workspace by its name.
    ///
    /// Returns a mutable reference.
    pub fn find_ws_mut(&mut self, name: &str) -> Option<&mut Workspace<P>> {
        self.workspaces.iter_mut()
            .find(|ws| ws.name == name)
    }

    //* Mutator and Manipulation Methods *//

    /// Cycle workspaces in given direction.
    pub fn cycle_to<C>(
        &mut self,
        scr: &Monitor<P>,
        cfg: &C,
        direction: Direction,
    )
    where
        C: RuntimeConfig,
    {
        // debug!("Cycling workspaces in direction {:?}", direction);
        // self.workspaces.cycle_focus(direction);

        // // i hate you, borrow checker
        // let name: String;
        // if let Some(ws) = self.workspaces.focused() {
        //     name = ws.name().into();
        // } else {
        //     return Err(OtherError("Focused should be Some".into()));
        // }
        // self.go_to(&name, scr, cfg)
    }

    /// Switch to a given workspace by its name.
    pub fn go_to<C>(&mut self, name: &str, scr: &Monitor<P>, cfg: &C)
    where
        C: RuntimeConfig,
    {
        // debug!("Going to workspace with name '{}'", name);

        // let new_idx = self
        //     .workspaces
        //     .index(Selector::Condition(&|ws| ws.name == name));
        // if new_idx.is_none() {
        //     return Err(UnknownWorkspace(name.into()));
        // }
        // let new_idx = new_idx.unwrap();
        // if self.current_idx() == new_idx {
        //     //todo: go to last workspace if same
        //     return Ok(());
        // }

        // self.current_mut().deactivate();
        // self.set_current(new_idx);

        // debug!("Goto workspace idx {}", new_idx);

        // if let Some(ws) = self.get_mut(self.current_idx()) {
        //     ws.activate(scr, cfg);
        // } else {
        //     return Err(UnknownWorkspace(name.into()));
        // }

        // Ok(())
    }
    /// Sends the currently focused window to the specified workspace.
    pub fn send_focused_to<C>(&mut self,name: &str, scr: &Monitor<P>, cfg: &C)
    where
        C: RuntimeConfig,
    {
        // debug!("Attempting to send window to workspace {}", name);
        // let winid = if let Some(window) = self.current().focused_client() {
        //     window.id()
        // } else {
        //     debug!("No focused window in workspace {}", name);
        //     return Ok(());
        // };
        // self.send_window_to(winid, name, scr, cfg)
    }

    /// Send a window to a given workspace.
    pub fn send_window_to<C>(&mut self, id: P::WindowId, name: &str, scr: &Monitor<P>, cfg: &C)
    where
        C: RuntimeConfig,
    {
        // debug!("Attempting to send window to workspace {}", name);
        // let Some(window) = self.current_mut().take_window(id) else {
        //     return Err(UnknownClient(id.clone()))
        // };
        // debug!("Sending window {:?} to workspace {}", window.id(), name);
        // let Some(ws) = self.find_mut(name) else {
        //     // if workspace was not found, put it back
        //     self.current_mut().put_window(window);
        //     return Err(UnknownWorkspace(name.into()))
        // };
        // ws.put_window(window);
        // if ws.focused_client().is_none() {
        //     ws.windows.set_focused_by_winid(id);
        // }
        // self.current_mut().relayout(scr, cfg);
        // Ok(())
    }
}

/// A workspace-screen multiplexer.
/// 
/// ## Convenient workspace-screen multiplexing
/// 
/// Unlike other compositors such as Hyprland or Niri, which have per-monitor workspaces,
/// `Toaru`'s window management model consists of a global namespace of workspaces, and 
/// outputs have a 'view' into that global namespace. Every active monitor displays one
/// monitor, and no two monitors can display the same workspace at any given moment
/// (unless one is a mirror of the other, which is a completely different scenario).
/// 
/// However, since all monitors share the same global namespace, changing the active workspace
/// on one monitor has to account for the positions of others, and maintaining internal
/// consistency between monitors in such a model is difficult when each monitor tracks its
/// own state individually.
/// 
/// `WorkspaceMux` fixes this problem by holding onto the global namespace of Workspaces, and
/// giving out [`Handles`](`WorkspaceMuxHandle`) to the namespace. Each handle exposes an
/// interface for working with `Workspace`s with respect to its own state, seamlessly handling
/// conflicts with other Handles transparently to the owner of the Handle.
#[derive(Debug)]
pub struct WorkspaceMux<P: Platform> {
    inner: Rc<WorkspaceMuxInner<P>>
}

impl<P: Platform> WorkspaceMux<P> {
    /// Creates a new `WorkspaceMux`.
    pub fn new<I>(workspaces: I) -> Self
    where
        I: IntoIterator<Item = Workspace<P>>
    {
        Self {
            inner: Rc::new(WorkspaceMuxInner::new(workspaces))
        }
    }

    /// Creates a new handle to a `WorkspaceMux`.
    /// 
    /// Returns `None` when there are no more open Workspaces.
    pub fn handle(&self) -> Option<WorkspaceMuxHandle<P>> {
        self.inner.add_token()
            .map(|idx| WorkspaceMuxHandle {
                token: idx,
                handle: Rc::clone(&self.inner)
            })
    }
}

// TODO: TESTING!!!

/// A handle to a workspace-screen multiplexer.
/// 
/// The `Handle` exposes an interface that allows you to operate on Workspaces
/// transparently to any other active `Handles`; the multiplexer handles conflict
/// checking and resolution for you.
/// 
/// When a `WorkspaceMuxHandle` is created, the next available `Workspace` (i.e.
/// one that is not currently being occupied by another `Handle`) is allocated to this
/// handle.
/// 
/// ## Conflict Resolution
/// 
/// Of course, since this is a *global* namespace of Workspaces that all outputs have
/// access to, there are still bound to be conflicts, such as cycling to a Workspace
/// that is already occupied. In these cases, the operation will be a no-op, unless
/// you pass in `swap=true` in the methods where it is available. In that case, the `Handle`
/// on the target workspace will swap places with your `Handle`.
/// 
#[derive(Debug)]
pub struct WorkspaceMuxHandle<P: Platform> {
    token: u64,
    handle: Rc<WorkspaceMuxInner<P>>
}

impl<P: Platform> WorkspaceMuxHandle<P> {
    /// Whether this `Handle` is registered with the `WorkspaceMux`.
    pub fn registered(&self) -> bool {
        self.handle.registered(self.token)
    }

    /// Register this `Handle` with the `WorkspaceMux`.
    /// 
    /// Returns `None` if already registered, or if no more handles can be registered.
    pub fn register(&self) -> Option<usize> {
        self.handle.register(self.token)
    }

    /// Deregister this `Handle` from the `WorkspaceMux`.
    pub fn deregister(&self) {
        self.handle.deregister(self.token)
    }

    /// Go to a workspace by name.
    pub fn goto_workspace_name(&self, name: &str, swap: bool) {
        self.handle.goto_workspace_name(name, self.token, swap);
    }

    /// Go to a workspace by index.
    pub fn goto_workspace_idx(&self, idx: usize, swap: bool) {
        self.handle.goto_workspace_idx(idx, self.token, swap);
    }

    /// Cycle to the workspace in the given direction.
    pub fn cycle_to_workspace(&self, dir: Direction, swap: bool) {
        self.handle.cycle_to_workspace(dir, self.token, swap);
    }

    /// Sends the current focused window to the Workspace with the matching `name`.
    pub fn send_focused_to(&self, name: &str) {
        self.handle.send_focused_to(name, self.token);
    }

    /// Run a closure with the current Workspace the handle is focused on.
    pub fn with_current<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Workspace<P>) -> T
    {
        self.handle.with_current(self.token, f)
    }
}

impl<P: Platform> PartialEq for WorkspaceMuxHandle<P> {
    fn eq(&self, other: &WorkspaceMuxHandle<P>) -> bool {
        self.token == other.token
    }
}

impl<P: Platform> Drop for WorkspaceMuxHandle<P> {
    fn drop(&mut self) {
        self.handle.remove_token(self.token);
    }
}

#[derive(Debug)]
struct WorkspaceMuxInner<P: Platform> {
    workspaces: RefCell<Vec<Workspace<P>>>,
    idxmap: RefCell<HashMap<u64, usize>>,
    token: Cell<u64>, // fixme: find a more sophisticated way of generating tokens
}

impl<P: Platform> WorkspaceMuxInner<P> {
    pub(crate) fn new<I: IntoIterator<Item = Workspace<P>>>(workspaces: I) -> Self {
        let workspaces = workspaces.into_iter().collect();

        Self {
            workspaces: RefCell::new(workspaces),
            idxmap: RefCell::new(HashMap::new()),
            token: Cell::new(1),
        }
    }

    /// Creates a new token and registers it with a new view into the Workspaces.
    pub(crate) fn add_token(&self) -> Option<u64> {
        let Some(next_idx) = self.next_avail_idx() else {
            return None
        };
        let new_token = self.token.get();
        self.token.set(new_token + 1);

        self.idxmap.borrow_mut().insert(new_token, next_idx);

        Some(new_token)
    }

    fn next_avail_idx(&self) -> Option<usize> {
        let size = self.workspaces.borrow().len();
        let taken = self.idxmap.borrow().values().map(|i| *i).collect::<Vec<_>>();

        (0..size).filter(|i| taken.contains(&i)).next()
    }

    pub(crate) fn remove_token(&self, token: u64) {
        self.idxmap.borrow_mut().remove(&token);
    }

    pub(crate) fn registered(&self, token: u64) -> bool {
        self.idxmap.borrow().contains_key(&token)
    }

    pub(crate) fn register(&self, token: u64) -> Option<usize> {
        if self.registered(token) {
            None
        } else if let Some(idx) = self.next_avail_idx() {
            self.idxmap.borrow_mut().insert(token, idx);
            Some(idx)
        } else {
            None
        }
    }

    pub(crate) fn deregister(&self, token: u64) {
        self.remove_token(token);
    }

    pub(crate) fn goto_workspace_name(&self, name: &str, token: u64, swap: bool) {
        todo!()
    }

    pub(crate) fn goto_workspace_idx(&self, idx: usize, token: u64, swap: bool) {
        todo!()
    }

    pub(crate) fn cycle_to_workspace(&self, dir: Direction, token: u64, swap: bool) {
        todo!()
    }

    pub(crate) fn send_focused_to(&self, name: &str, token: u64) {
        todo!()
    }

    pub(crate) fn current_idx(&self, token: u64) -> Option<usize> {
        self.idxmap.borrow().get(&token).map(|i| *i)
    }

    pub(crate) fn with_current<F, T>(&self, token: u64, f: F) -> T
    where
        F: FnOnce(&mut Workspace<P>) -> T
    {
        let idx = *self.idxmap.borrow().get(&token).unwrap();

        f(&mut self.workspaces.borrow_mut()[idx])
        
    }
}
