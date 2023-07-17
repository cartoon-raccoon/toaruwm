//! Types for working with the internal state of a `WindowManager`.
//! 
//! This module contains types and expose the internal state of a
//! `WindowManager`, and also provides traits to allow you
//! to call [`Workspace`] and [`Desktop`] methods with your own
//! types.

use std::collections::HashMap;
use std::any::Any;

use custom_debug_derive::Debug;

use crate::core::{
    types::Color, Client, Desktop, Ring, Workspace,
};
use crate::x::{XConn, XWindow, XWindowID};

/// An object that can provide information about window manager state
/// at runtime.
/// 
/// This trait allows you to create objects representing current
/// `WindowManager` state and configuration. It is passed to various
/// [`Workspace`] and [`Desktop`] methods to allow then to account for
/// various configuration details when executing their functionality.
/// 
/// As this trait is used as a trait object during the window manager
/// runtime, its methods cannot be generic.
/// 
/// # Retrieving Arbitrary Values
/// 
/// One of `RuntimeConfig`'s required methods is `get_key`, which
/// returns a dynamically typed trait object (i.e. `&dyn Any`).
/// 
/// It is then up to the caller to see if this object is of the
/// needed type, by calling `downcast_ref` on it.
/// 
/// ## Example
/// 
/// ```rust
/// //todo
/// ```
pub trait RuntimeConfig {
    /// Return information about the floating classes.
    fn float_classes(&self) -> &[String];

    /// Return information about the window border thickness.
    fn border_px(&self) -> u32;

    /// Return information about unfocused window border color.
    fn unfocused(&self) -> Color;

    /// Return information about focused window border color.
    fn focused(&self) -> Color;
    
    /// Return information about urgent window border color.
    fn urgent(&self) -> Color;

    /// Retrieve arbitrary key value pairs from storage.
    /// 
    /// Should return None if the key does not exist in
    /// storage.
    fn get_key(&self, key: &str) -> Option<&dyn Any>;
}

/// The runtime configuration of the 
/// [`WindowManager`](super::WindowManager).
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
pub struct WmConfig {
    pub(crate) float_classes: Vec<String>,
    pub(crate) border_px: u32,
    pub(crate) unfocused: Color,
    pub(crate) focused: Color,
    pub(crate) urgent: Color,
    pub(crate) keys: HashMap<String, Box<dyn Any>>,
}

impl RuntimeConfig for WmConfig {
    fn float_classes(&self) -> &[String] {
        &self.float_classes
    }

    fn border_px(&self) -> u32 {
        self.border_px
    }

    fn unfocused(&self) -> Color {
        self.unfocused
    }

    fn focused(&self) -> Color {
        self.focused
    }

    fn urgent(&self) -> Color {
        self.urgent
    }

    fn get_key(&self, key: &str) -> Option<&dyn Any> {
        self.keys.get(&String::from(key)).map(|v| &*v as &dyn Any)
    }
}

/// The state that the current window manager is in.
#[non_exhaustive]
#[derive(std::fmt::Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum State {}

/// Provides introspection into the state of the window manager.
///
/// The `'wm` lifetime refers to the lifetime of the parent
/// `WindowManager` type.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct WmState<'wm, X, C>
where
    X: XConn,
    C: RuntimeConfig
{
    /// The `XConn` implementation currently being used.
    #[debug(skip)]
    pub conn: &'wm X,
    /// The inner configuration of the WindowManager.
    pub config: &'wm C,
    /// The workspaces maintained by the window manager.
    pub workspaces: &'wm Ring<Workspace>,
    /// The root window.
    pub root: XWindow,
    /// The selected window, if any.
    pub selected: Option<XWindowID>,
    pub(crate) desktop: &'wm Desktop,
}

impl<'wm, X, C> WmState<'wm, X, C>
where
    X: XConn,
    C: RuntimeConfig,
{
    /// Looks up a client with the given X ID.
    pub fn lookup_client(&self, id: XWindowID) -> Option<&Client> {
        self.desktop.current().windows.lookup(id)
    }

    /// Checks whether the window `id` is currently managed.
    pub fn is_managing(&self, id: XWindowID) -> bool {
        self.desktop.is_managing(id)
    }
}
