//! Types used for aggregate workspace management.
//!
//! This module exports [`Monitor`] and [`Workspaces`], as well as [`WorkspaceMux`], which
//! implement functionality for managing and displaying workspaces.
//!
//! A [`Monitor`] represents a physical monitor that is connected to your computer.
//! It encapsulates monitor resolution and is used by the tiling algorithms to resize windows,
//! among other things.
//! 
//! A `Monitor` internally tracks its state using a handle to the platform-defined output, and
//! with a [`Handle`](WorkspaceMuxHandle) to a [`WorkspaceMux`], which manages which `Monitor`s
//! are displaying what [`Workspace`]s.
//! 
//! See the module-level docs for more information, specifically the documentation of
//! [`WorkspaceMux`] for information on Toaru's workspace management model.

use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use tracing::trace;

use crate::core::Workspace;
use crate::config::RuntimeConfig;
use crate::ToaruError;
use crate::platform::{Platform, PlatformOutput};
use crate::types::{
    Cardinal, Direction, Rectangle, Logical
};
use crate::Result;

/// A physical monitor that can display a workspace.
/// 
/// In order to make workspace management easier and more predictable, there one invariant
/// that defines the relationship between `Monitor`s and `Workspace`s:
/// 
/// > Every monitor must have a workspace to display. If there are more monitors than workspaces,
/// workspaces will be automatically created to maintain this invariant.
/// 
/// At any point, if this invariant is violated, a panic will be triggered.
#[derive(Debug, Clone)]
pub struct Monitor<P: Platform> {
    pub(crate) name: String,
    pub(crate) output: P::Output,
    pub(crate) workspace_handle: WorkspaceMuxHandle<P>,
    /// The usable geometry of the Screen.
    pub(crate) effective_geom: Rectangle<i32, Logical>,
    /// The index of the Screen.
    pub(crate) idx: i32,
}

impl<P: Platform> Monitor<P> {
    /// Creates a new Monitor with the provided output and workspace handle.
    pub fn new(output: P::Output, workspace_handle: WorkspaceMuxHandle<P>, screen_idx: i32) -> Self {
        let effective_geom = output.geometry().unwrap_or_else(|| Rectangle::zeroed());
        Self {
            name: output.name(),
            output,
            workspace_handle,
            effective_geom,
            idx: screen_idx,
        }
    }

    /// Returns the name of the Monitor.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the index of the Monitor.
    pub fn idx(&self) -> i32 {
        self.idx
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

    /// Returns a reference to the [`WorkspaceMuxHandle`] owned by the Monitor.
    pub fn handle(&self) -> &WorkspaceMuxHandle<P> {
        &self.workspace_handle
    }
}

/// An namespace of [Workspace]s.
/// 
/// This struct wraps a collection of [`Workspace`]s, and exposes safe methods to manipulate
/// the global `Workspace` namespace.
/// 
/// All `Workspace`s exist in a global namespace that all monitors have access to. Therefore,
/// there can be no name collisions between workspaces, and `Workspaces` helps to enforce this
/// invariant.
/// 
#[derive(Debug)]
pub struct Workspaces<P: Platform> {
    wksps: Vec<Workspace<P>>,
    names: HashSet<String>
}

impl<P: Platform> Workspaces<P> {
    /// Creates a new `Workspace` namespace.
    pub fn new<I>(workspaces: I) -> Result<Self>
    where
        I: IntoIterator<Item = Workspace<P>>
    {
        let mut wksps = Vec::new();
        let mut names = HashSet::new();
        
        for wksp in workspaces.into_iter() {
            names.insert(wksp.name().to_string());
            wksps.push(wksp);
        }

        if names.len() != wksps.len() {
            wksps.retain(|ws| !names.contains(ws.name()));

            let ret: Vec<String> = wksps.into_iter().map(|ws| ws.name).collect();

            Err(ToaruError::NamespaceConflict(ret.join(", ")))
        } else {
            Ok(Self { wksps, names })
        }
    }

    /// Creates a new workspace, adding it if it does not violate the invariants of the namespace.
    /// 
    /// If the workspace was successfully added, `None` is returned, otherwise the created workspace
    /// is added.
    pub fn add_workspace<S: Into<String>>(&mut self, name: S, output: Option<P::Output>) -> Option<Workspace<P>> {
        let name: String = name.into();
        let new: Workspace<P> = if let Some(output) = output {
            Workspace::new_with_output(&name, output)
        } else {
            Workspace::new(&name)
        };

        if self.contains_name(&name) {
            Some(new)
        } else {
            self.wksps.push(new);
            None
        }
    }

    /// Removes the workspace with the given name, returning it if it exists.
    pub fn del_workspace<S: AsRef<str>>(&mut self, name: S) -> Option<Workspace<P>> {
        todo!()
    }

    /// Checks whether the name provided already exists in the namespace defined by this `Workspaces`.
    pub fn contains_name<S: AsRef<str>>(&self, name: S) -> bool {
        self.names.contains(name.as_ref())
    }
}

impl<P: Platform> Deref for Workspaces<P> {
    type Target = [Workspace<P>];

    fn deref(&self) -> &[Workspace<P>] {
        &self.wksps
    }
}

impl<P: Platform> DerefMut for Workspaces<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.wksps
    }
}

/// A type to select a workspace, either by index or by name.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WsSelector<'a> {
    /// Select the workspace by index.
    Index(usize),
    /// Select the workspace by name.
    Name(&'a str),
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
/// 
/// ## Handle-Workspace Overflow
/// 
/// If too many monitors are connected, it is possible that there will be more handles created
/// than there are workspaces to occupy. In such cases, when a new Handle is created that
/// would cause an overflow, the [`WorkspaceMux`] will attempt to create a new Workspace with
/// the name being `number of existing handles + 1` (e.g. if there are currently 4 handles it
/// will create a workspace with name "5"). If this too fails (e.g. because it would violate 
/// the namespace invariants) it will then **panic**.
#[derive(Debug)]
pub struct WorkspaceMux<P: Platform> {
    inner: Rc<WorkspaceMuxInner<P>>
}

impl<P: Platform> WorkspaceMux<P> {
    /// Creates a new `WorkspaceMux`.
    pub fn new<I>(workspaces: I) -> Result<Self>
    where
        I: IntoIterator<Item = Workspace<P>>
    {
        Ok(Self {
            inner: Rc::new(WorkspaceMuxInner::new(workspaces)?)
        })
    }

    /// Creates a new handle to a `WorkspaceMux`.
    /// 
    /// ## Panics
    /// 
    /// This function will panic if creating this handle would cause an overflow
    /// (see above) and attempting to fix the overflow by creating a new workspace
    /// fails.
    pub fn handle(&self, output: &P::Output) -> WorkspaceMuxHandle<P> {
        let token = self.inner.add_token(output);

        WorkspaceMuxHandle {
            token,
            handle: Rc::downgrade(&self.inner),
        }
    }

    /// Runs a closure on all the workspaces in the Multiplexer.
    pub fn with_workspaces<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut [Workspace<P>]) -> T
    {
        let mut workspaces = self.inner.workspaces.borrow_mut();

        f(&mut workspaces)
    }

    /// Checks if the provided `token` is currently registered with the `WorkspaceMux`.
    pub fn token_is_registered(&self, token: u64) -> bool {
        self.inner.registered(token)
    }

    /// Checks if the window with the given `id` is inside any workspace.
    pub fn is_managing(&self, id: P::WindowId) -> bool {
        self.inner.is_managing(id)
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
/// If the underlying [`WorkspaceMux`] is dropped while this Handle is still active,
/// all method calls on the handle will fail.
/// 
/// ## Conflict Resolution
/// 
/// Of course, since this is a *global* namespace of Workspaces that all outputs have
/// access to, there are still bound to be conflicts, such as cycling to a Workspace
/// that is already occupied. In these cases, the operation will be a no-op, unless
/// you pass in `swap=true` in the methods where it is available. In that case, the `Handle`
/// on the target workspace will swap places with your `Handle`.
#[derive(Debug)]
pub struct WorkspaceMuxHandle<P: Platform> {
    token: u64,
    handle: Weak<WorkspaceMuxInner<P>>
}

impl<P: Platform> WorkspaceMuxHandle<P> {
    /// Whether this `Handle` is registered with an underlying `WorkspaceMux`.
    pub fn registered(&self) -> bool {
        self.handle.upgrade()
            .map(|h| h.registered(self.token))
            .is_some_and(|b| b)
    }

    /// Register this `Handle` with its underlying `WorkspaceMux`.
    /// 
    /// Returns `None` if already registered, or if no more handles can be registered.
    pub fn register(&self) -> Option<usize> {
        self.handle.upgrade()
            .and_then(|h| h.register(self.token))
    }

    /// Deregister this `Handle` from the `WorkspaceMux`.
    pub fn deregister(&self) {
        self.handle.upgrade()
            .map(|h| h.deregister(self.token));
    }

    /// Checks if a window with a given `id` is being managed by any of the workspaces.
    pub fn is_managing(&self, id: P::WindowId) -> bool {
        self.handle.upgrade()
            .map(|h| h.is_managing(id))
            .is_some_and(|b| b)
    }

    /// Go to a workspace, either by name or by index.
    pub fn go_to(&self, sel: WsSelector<'_>, swap: bool) -> Option<usize> {
        let Some(handle) = self.handle.upgrade() else {
            return None
        };
        match sel {
            WsSelector::Index(idx) => handle.goto_workspace_idx(idx, self.token, swap),
            WsSelector::Name(name) => handle.goto_workspace_name(name, self.token, swap),
        }
    }

    /// Cycle to the workspace in the given direction.
    pub fn cycle_to(&self, dir: Direction, swap: bool, wrap: bool) -> Option<usize> {
        self.handle.upgrade()
            .and_then(|h| h.cycle_to(dir, self.token, swap, wrap))
    }

    /// Sends the current focused window to the Workspace with the matching `name`.
    pub fn send_focused_to(&self, sel: WsSelector<'_>) -> bool {
        let Some(handle) = self.handle.upgrade() else {
            return false
        };

        match sel {
            WsSelector::Index(idx) => handle.send_focused_to_idx(idx, self.token),
            WsSelector::Name(name) => handle.send_focused_to_name(name, self.token),
        }
    }

    /// Sends the window with the given `id` to the workspace, either by name or by index.
    pub fn send_window_to(&self, id: P::WindowId, sel: WsSelector<'_>) -> bool {
        let Some(handle) = self.handle.upgrade() else {
            return false;
        };

        match sel {
            WsSelector::Index(idx) => handle.send_window_to_idx(id, idx, self.token),
            WsSelector::Name(name) => handle.send_window_to_name(id, name, self.token),
        }
    }

    /// Returns the index of the current workspace, if any.
    /// 
    /// Returns None if the `Handle` is not currently assigned to a workspace,
    /// or if the underlying `WorkspaceMux` was dropped.
    pub fn current_idx(&self) -> Option<usize> {
        self.handle.upgrade()
            .and_then(|h| h.current_idx(self.token))
    }

    /// Run a closure with the current Workspace the handle is focused on.
    /// 
    /// Returns None if the `Handle` is not currently assigned to a workspace,
    /// or if the underlying `WorkspaceMux` was dropped.
    pub fn with_current<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Workspace<P>) -> T
    {
        self.handle.upgrade()
            .and_then(|h| h.with_current(self.token, f))
    }
}

impl<P: Platform> PartialEq for WorkspaceMuxHandle<P> {
    fn eq(&self, other: &WorkspaceMuxHandle<P>) -> bool {
        self.token == other.token
    }
}

impl<P: Platform> Drop for WorkspaceMuxHandle<P> {
    fn drop(&mut self) {
        self.handle.upgrade()
            .inspect(|h| h.remove_token(self.token));
    }
}

impl<P: Platform> Clone for WorkspaceMuxHandle<P> {
    fn clone(&self) -> Self {
        let handle = match &self.handle.upgrade() {
            Some(h) => Rc::downgrade(h),
            None => Weak::new()
        };

        Self {
            token: self.token,
            handle
        }
    }
}

#[derive(Debug)]
struct WorkspaceMuxInner<P: Platform> {
    /// The overall 
    workspaces: RefCell<Workspaces<P>>,
    /// The mapping of tokens to their currently assigned workspace, by index.
    idxmap: RefCell<HashMap<u64, usize>>,
    /// The next token to give out.
    next_token: Cell<u64>, //? Is simply counting up a good way to give out tokens?
}

impl<P: Platform> WorkspaceMuxInner<P> {
    pub(crate) fn new<I: IntoIterator<Item = Workspace<P>>>(workspaces: I) -> Result<Self> {
        let workspaces = Workspaces::new(workspaces)?;

        Ok(Self {
            workspaces: RefCell::new(workspaces),
            idxmap: RefCell::new(HashMap::new()),
            next_token: Cell::new(1),
        })
    }

    /// Creates a new token and registers it with a new view into the Workspaces.
    pub(crate) fn add_token(&self, output: &P::Output) -> u64 {
        if let Some(next_idx) = self.next_avail_idx() {
            self.add_token_unchecked(next_idx)
        } else {
            // if no index is available, we've hit an overflow
            // create a new workspace and then add it to the inner workspaces.
            let new_ws_name = self.idxmap.borrow().len().to_string();
            trace!("Creating new workspace with name {new_ws_name}");
            let res = self.workspaces.borrow_mut().add_workspace(&new_ws_name, Some(output.clone()));

            // This will trigger a panic on a namespace conflict, which we want.
            assert!(res.is_none());

            let next_idx = self.next_avail_idx().expect("should have an available idx");

            self.add_token_unchecked(next_idx)
        }
    }

    fn add_token_unchecked(&self, idx: usize) -> u64 {
        let new_token = self.next_token.get();
        self.next_token.set(new_token + 1);

        assert!(self.idxmap.borrow_mut().insert(new_token, idx).is_none());

        new_token
    }

    fn next_avail_idx(&self) -> Option<usize> {
        let size = self.workspaces.borrow().len();
        let taken = self.idxmap.borrow().values().map(|i| *i).collect::<Vec<_>>();

        (0..size).filter(|i| taken.contains(i)).next()
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

    pub(crate) fn is_managing(&self, id: P::WindowId) -> bool {
        self.workspaces.borrow().iter().any(|ws| ws.contains_window(id))
    }

    pub(crate) fn goto_workspace_name(&self, name: &str, token: u64, swap: bool) -> Option<usize> {
        // get the idx of the workspace to swap to. if no such workspace exists, return immediately.
        let target_idx = self.workspaces.borrow()
            .iter()
            .enumerate()
            .find(|(_, ws)| ws.name() == name)
            .map(|(idx, _)| idx)?;

        self.goto_workspace_idx(target_idx, token, swap)
    }

    pub(crate) fn goto_workspace_idx(&self, idx: usize, token: u64, swap: bool) -> Option<usize> {
        if let Some((occ_tok, _)) = self.idx_is_occupied(idx) {
            // the target workspace is currently occupied
            if !swap {
                // if swap is not specified, then do nothing and return immediately
                return None
            }

            if !self.swap(token, occ_tok) {
                return None
            }
        } else {
            // the target workspace is currently unoccupied, switch directly
            *(self.idxmap.borrow_mut().get_mut(&token).unwrap()) = idx;
        }

        //todo: reconfigure the workspace if active

        Some(idx)
    }

    pub(crate) fn cycle_to(&self, dir: Direction, token: u64, swap: bool, wrap: bool) -> Option<usize> {
        let current_idx = self.current_idx(token)?;
        let max = self.workspaces.borrow().len();

        let new_idx = match dir {
            // would wrap to the front
            Direction::Forward if current_idx + 1 >= max => {
                if !wrap {
                    return None
                } else {
                    0
                }
            },
            // would wrap to the back
            Direction::Backward if current_idx == 0 => {
                if !wrap {
                    return None
                } else {
                    max - 1
                }
            }
            Direction::Forward => current_idx + 1,
            Direction::Backward => current_idx - 1,
        };

        self.goto_workspace_idx(new_idx, token, swap)
    }

    pub(crate) fn send_focused_to_name(&self, name: &str, token: u64) -> bool {
        todo!()
    }

    pub(crate) fn send_focused_to_idx(&self, idx: usize, token: u64) -> bool {
        todo!()
    }

    pub(crate) fn send_window_to_name(&self, id: P::WindowId, name: &str, token: u64) -> bool {
        todo!()
    }

    pub(crate) fn send_window_to_idx(&self, id: P::WindowId, idx: usize, token: u64) -> bool {
        todo!()
    }

    pub(crate) fn current_idx(&self, token: u64) -> Option<usize> {
        self.idxmap.borrow().get(&token).map(|i| *i)
    }

    pub(crate) fn with_current<F, T>(&self, token: u64, f: F) -> Option<T>
    where
        F: FnOnce(&mut Workspace<P>) -> T
    {
        let Some(idx) = self.idxmap.borrow().get(&token).map(|v| *v) else {
            return None
        };

        Some(f(&mut self.workspaces.borrow_mut()[idx]))
        
    }

    /// Checks if the workspace at the current index is occupied, and if so, by which handle.
    fn idx_is_occupied(&self, idx: usize) -> Option<(u64, usize)> {
        self.idxmap.borrow().iter()
            .find(|(_, v)| **v == idx)
            .map(|(k, v)| (*k, *v))
    }

    /// Swaps the assigned indexes of two tokens.
    fn swap(&self, tok1: u64, tok2: u64) -> bool {

        let Some(idx1) = self.idxmap.borrow().get(&tok1).map(|idx| *idx) else {
            return false
        };
        let Some(idx2) = self.idxmap.borrow().get(&tok2).map(|idx| *idx) else {
            return false
        };

        *(self.idxmap.borrow_mut().get_mut(&tok1).unwrap()) = idx2;
        *(self.idxmap.borrow_mut().get_mut(&tok2).unwrap()) = idx1;
        
        true
    }
}

#[cfg(test)]
mod test {
    //use test_log::test;
    
}
