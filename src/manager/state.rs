//! Types for working with the internal state of a `WindowManager`.
//! 
//! This module contains types and expose the internal state of a
//! `WindowManager`, and also provides traits to allow you
//! to call [`Workspace`] and [`Desktop`] methods with your own
//! types.

use std::collections::HashMap;
use std::any::Any;

use custom_debug_derive::Debug;

use super::WindowManager;
use crate::core::{types::Color, Client, Desktop, Ring, Workspace};
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
    fn get_key(&self, key: &str) -> Option<&dyn Any>;
}

/// The runtime configuration of the [`WindowManager`].
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
    /// Get floating classes.
    fn float_classes(&self) -> &[String] {
        &self.float_classes
    }

    /// Get the border thickness.
    fn border_px(&self) -> u32 {
        self.border_px
    }

    /// Get the border color of unfocused windows.
    fn unfocused(&self) -> Color {
        self.unfocused
    }

    /// Get the border color of focused windows.
    fn focused(&self) -> Color {
        self.focused
    }

    /// Get the border color of urgent windows.
    fn urgent(&self) -> Color {
        self.urgent
    }

    /// Get an arbitrary key-value pair.
    /// 
    /// Returns None if the value does not exist or
    /// is not of the type specified.
    /// 
    /// See the `get_key` method
    /// in [`Config`](crate::manager::Config)
    /// for more details on how to use this function.
    fn get_key(&self, key: &str) -> Option<&dyn Any>
    {
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
pub struct WmState<'wm, X: XConn> {
    /// The `XConn` implementation currently being used.
    #[debug(skip)]
    pub conn: &'wm X,
    /// The inner configuration of the WindowManager.
    pub config: &'wm WmConfig,
    /// The workspaces maintained by the window manager.
    pub workspaces: &'wm Ring<Workspace>,
    /// The root window.
    pub root: XWindow,
    /// The selected window, if any.
    pub selected: Option<XWindowID>,
    pub(crate) desktop: &'wm Desktop,
}

//todo: implement debug!

impl<X: XConn> WindowManager<X> {
    /// Provides a WMState for introspection.
    pub fn state(&self) -> WmState<'_, X> {
        WmState {
            conn: &self.conn,
            config: &self.config,
            workspaces: &self.desktop.workspaces,
            desktop: &self.desktop,
            root: self.root,
            selected: self.selected,
        }
    }
}

impl<'wm, X: XConn> WmState<'wm, X> {
    /// Looks up a client with the given X ID.
    pub fn lookup_client(&self, id: XWindowID) -> Option<&Client> {
        self.desktop.current().windows.lookup(id)
    }

    /// Checks whether the window `id` is currently managed.
    pub fn is_managing(&self, id: XWindowID) -> bool {
        self.desktop.is_managing(id)
    }
}
