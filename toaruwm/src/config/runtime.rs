//! Runtime configuration for a `Toaru` instance.

use std::fmt::Debug;

use crate::config::{OutputLayout, ConfigSection};
use crate::platform::{
    x11::X11Config,
    wayland::WaylandConfig,
};

/// An object that can provide information about your
/// configuration at runtime.
///
/// This trait allows you to create objects representing current
/// `Toaru` state and configuration. It is passed to various
/// [`Workspace`] and [`Desktop`] methods to allow then to account for
/// various configuration details when executing their functionality.
///
/// As this trait is used as a trait object during the window manager
/// runtime, its methods cannot be generic.
/// 
/// # Retrieving platform-specific configuration
/// 
/// There are provided methods, [`wayland_cfg`][3], and [`x11_cfg`][4],
/// to optionally return platform-specific configuration objects.
/// Re-implement them if you want to customize your platform configuration,
/// otherwise sensible defaults will be chosen.
///
/// [3]: RuntimeConfig::wayland_cfg
/// [4]: RuntimeConfig::x11_cfg
pub trait RuntimeConfig: Debug {
    /// Return information about the floating classes.
    fn float_classes(&self) -> &[String];

    /// Return information about the gaps between windows.
    fn window_gap(&self) -> u32;

    /// Return whether the focus should follow the pointer.
    fn focus_follows_ptr(&self) -> bool;

    /// Return the outputs and their layout.
    fn outputs(&self) -> &OutputLayout;

    /// Get a Config section by name.
    fn section(&self, name: &str) -> Option<&dyn ConfigSection>;

    /// Returns Wayland-specific configuration.
    #[cfg(feature = "wayland-core")]
    fn wayland(&self) -> &dyn WaylandConfig;

    /// Returns X11-specific configuration.
    #[cfg(feature = "x11-core")]
    fn x11(&self) -> &dyn X11Config;
}