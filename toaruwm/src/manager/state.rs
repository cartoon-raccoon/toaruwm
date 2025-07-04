//! Types for working with the internal state of a `WindowManager`.
//!
//! This module contains types and expose the internal state of a
//! `WindowManager`, and also provides traits to allow you
//! to call [`Workspace`] and [`Desktop`] methods with your own
//! types.

use crate::config::{OutputLayout, RuntimeConfig, ConfigSection};
use crate::platform::{
    wayland::{WaylandConfig, ToaruWaylandConfig}, 
    x11::{X11Config, ToaruX11Config},
};
use crate::core::{WorkspaceMuxHandle};
use crate::Platform;

/// The an implementation of runtime configuration for 
/// [`Toaru`](super::Toaru).
///
/// Since a user-created [`Config`](crate::manager::Config)
/// has several fields moved out of it during window manager
/// initialization, this contains the remaining fields
/// that are used by the rest of the window manager's functionality.
///
/// It is not possible for users to construct this type directly,
/// as it is owned by `WindowManager` and is constructed internally
/// on initialization. It is only passed to user code to provide
/// various configuration details that may be needed for such code
/// to work.
///
/// This type implements `RuntimeConfig`.
#[derive(Debug)]
pub struct ToaruRuntimeConfig {
    pub(crate) float_classes: Vec<String>,
    pub(crate) window_gap: u32,
    pub(crate) focus_follows_ptr: bool,
    pub(crate) outputs: OutputLayout,
    pub(crate) waylandcfg: ToaruWaylandConfig,
    pub(crate) x11cfg: ToaruX11Config,
}

impl RuntimeConfig for ToaruRuntimeConfig {
    fn float_classes(&self) -> &[String] {
        &self.float_classes
    }

    fn window_gap(&self) -> u32 {
        self.window_gap
    }

    fn focus_follows_ptr(&self) -> bool {
        self.focus_follows_ptr
    }

    fn outputs(&self) -> &OutputLayout {
        &self.outputs
    }

    fn section(&self, name: &str) -> Option<&dyn ConfigSection> {
        None // todo
    }

    fn wayland(&self) -> &dyn WaylandConfig {
        &self.waylandcfg
    }

    fn x11(&self) -> &dyn X11Config {
        &self.x11cfg
    }
}

/// The state that the current window manager is in.
#[non_exhaustive]
#[derive(std::fmt::Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum State {}

/// Provides introspection into the state of the running `Toaru` instance.
///
/// The `'t` lifetime refers to the lifetime of the parent
/// `Toaru` type.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct ToaruState<'t, P, C>
where
    P: Platform,
    C: RuntimeConfig
{
    /// The inner configuration of the WindowManager.
    pub config: &'t C,
    /// The workspaces maintained by the window manager.
    pub workspaces: &'t WorkspaceMuxHandle<P>,
    /// The selected window, if any.
    pub selected: Option<&'t P::WindowId>,
}

impl<'t, P, C> ToaruState<'t, P, C>
where
    P: Platform,
    C: RuntimeConfig,
{
    /// Checks whether the window `id` is currently managed.
    pub fn is_managing(&self, id: P::WindowId) -> bool {
        todo!()
    }
}
