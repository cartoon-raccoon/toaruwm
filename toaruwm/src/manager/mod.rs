//! The window manager itself, and associated modules.

#![allow(unused_variables)] //fixme

/// Macros and storage types for window manager hooks.
pub mod hooks;
pub mod state;

mod manager;

#[doc(inline)]
pub use manager::Toaru;

#[doc(inline)]
pub use hooks::{Hook, Hooks};
#[doc(inline)]
pub use state::ToaruState;

use crate::platform::Platform;
use crate::config::RuntimeConfig;
use crate::types::{Rectangle, Logical, Direction};
use crate::core::{Window, Monitor, Workspace};

/// A type that implements window management functionality.
/// 
/// This is a supertrait for the [`ManagerCommandHandler`] and [`ManagerPlatformInterface`],
/// which define the functionality a `Manager` must implement: Expose an interface that
/// a platform can use to manipulate its inner state and get window data; and process
/// commands from users to manipulate the windows in its current state.
pub trait Manager<P: Platform>: ManagerCommandHandler + ManagerPlatformInterface<P> {}

/// A type that can handle commands sent to a Manager.
pub trait ManagerCommandHandler {
    /// Go to the specified workspace.
    fn goto_workspace(&mut self, name: &str);

    /// Cycles the focused workspace.
    fn cycle_workspace(&mut self, name: &str);

    /// Sends the focused window to the currently active monitor.
    fn send_focused_to(&mut self, name: &str, switch: bool);

    /// Cycles the focused window.
    fn cycle_focus(&mut self, direction: Direction);

    /// Cycles in the given direction to the layout applied to the current workspace.
    fn cycle_layout(&mut self, direction: Direction);

    /// Toggles the state of the focused window to floating or vice versa.
    fn toggle_focused_floating(&mut self);

    /// Sends an [`Update`](crate::layouts::update::Update)
    /// to the current layout.
    fn update_current_layout(&mut self);
}

/// A type that implements an interface with a [`Platform`].
/// 
/// This trait defines the interface that a [`Manager`] must expose to allow a [`Platform`]
/// to manipulate the windows it manages, and for a platform to get the configuration
/// of windows that it manages, in order to display them to the screen.
pub trait ManagerPlatformInterface<P: Platform> {

    /// The runtime configuration of the Manager.
    type Config: RuntimeConfig;

    /// Returns a reference to the internal runtime configuration.
    fn config(&self) -> &Self::Config;

    /// Add an output to the Manager..
    fn add_output(&mut self, output: P::Output);

    /// Gets the monitor corresponding to the platform specific output.
    fn get_output(&mut self, output: &P::Output) -> Option<&mut Monitor<P>>;

    /// Remove an output from the Manager.
    fn remove_output(&mut self, output: &P::Output) -> Option<Monitor<P>>;

    /// Insert a new window into the Manager.
    fn insert_window(&mut self, id: P::WindowId, output: Option<&P::Output>);

    /// Run a closure on the window with `id`.
    fn with_window<F, T>(&mut self, id: P::WindowId, f: F) -> T
    where
        F: FnOnce(&mut Window<P>) -> T;

    /// Remove a window identified by `id`.
    fn remove_window(&mut self, id: P::WindowId) -> Option<Window<P>>;

    /// Configures a window with `id` with `geom`.
    fn configure_window(&mut self, id: P::WindowId, geom: Rectangle<i32, Logical>);

    /// Map the window with `id`.
    fn map_window(&mut self, id: P::WindowId);

    /// Unmap
    fn unmap_window(&mut self, id: P::WindowId);

    /// Runs a closure on all workspaces managed within a Manager.
    fn with_workspaces<F, T>(&mut self, f: F) -> T 
    where
        F: FnOnce(&mut [Workspace<P>]) -> T;

    /// Run a closure on for each workspace managed within a Manager.
    /// 
    /// If `active_only` is true, the closure is run only for active workspaces.
    fn foreach_workspace<F>(&mut self, active_only: bool, f: F)
    where
        F: FnMut(&mut Workspace<P>);

}