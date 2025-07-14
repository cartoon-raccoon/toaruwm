//! Runtime configuration for a `Toaru` instance.
//! 
//! This module exports the [`RuntimeConfig`] trait, the core trait that allows a type to provide
//! configuration info at runtime, as well as [`ManagerConfig`], a type-erased, easily clonable,
//! handle to an underlying implementor of [`RuntimeConfig`].

use std::fmt::Debug;
use std::any::Any;
//use std::convert::AsRef;
use std::rc::Rc;

use crate::config::{OutputLayout, ConfigSection};
use crate::platform::{
    x11::X11Config,
    wayland::WaylandConfig,
};

/// A type-erased wrapper around a RuntimeConfig.
/// 
/// A `ManagerConfig` wraps an implementor of `RuntimeConfig`, allowing its
/// type parameter to be erased, but also giving you the flexibility to retrieve
/// a reference to its underlying type, if needed.
/// 
/// Internally, a `ManagerConfig` stores its wrapped value in an `Rc`, allowing you to
/// shallow clone it and pass owned values around wherever you might need it.
#[derive(Debug)]
pub struct MgrConfig {
    inner: Rc<Box<dyn RuntimeConfig>>
}

impl MgrConfig {
    /// Wraps a RuntimeConfig type, returning a new ManagerConfig.
    pub fn new<C: RuntimeConfig + 'static>(config: C) -> Self {
        Self {
            inner: Rc::new(Box::new(config) as Box<dyn RuntimeConfig>)
        }
    }

    /// Attempts to downcast the Manager config as a concrete type.
    pub fn try_downcast<C: RuntimeConfig>(&self) -> Option<&C> {
        (self.inner.as_ref() as &dyn Any).downcast_ref::<C>()
    }

    /// Downcasts the `ManagerConfig` as a concrete type, panicking if the type parameter is not correct.
    pub fn downcast<C: RuntimeConfig>(&self) -> &C {
        self.try_downcast().expect("inner type does not match inserted type param")
    }
}

impl Clone for MgrConfig {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner)
        }
    }
}

impl RuntimeConfig for MgrConfig {
    fn float_classes(&self) -> &[String] {
        self.inner.float_classes()
    }

    fn window_gap(&self) -> u32 {
        self.inner.window_gap()
    }

    fn focus_follows_ptr(&self) -> bool {
        self.inner.focus_follows_ptr()
    }

    fn outputs(&self) -> &OutputLayout {
        self.inner.outputs()
    }

    fn section(&self, name: &str) -> Option<&dyn ConfigSection> {
        self.inner.section(name)
    }

    #[cfg(feature = "wayland-core")]
    fn wayland(&self) -> &dyn WaylandConfig {
        self.inner.wayland()
    }

    #[cfg(feature = "x11-core")]
    fn x11(&self) -> &dyn X11Config {
        self.inner.x11()
    }
}

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
pub trait RuntimeConfig: Debug + Any {
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

    /// Wraps the `RuntimeConfig` in a new `ManagerConfig`.
    fn into_managerconfig(self) -> MgrConfig
    where
        Self: Sized
    {
        MgrConfig::new(self)
    }
}