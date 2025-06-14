//! Types for working with the internal state of a `WindowManager`.
//!
//! This module contains types and expose the internal state of a
//! `WindowManager`, and also provides traits to allow you
//! to call [`Workspace`] and [`Desktop`] methods with your own
//! types.

use std::any::Any;
use std::collections::HashMap;

use custom_debug_derive::Debug;

use crate::core::{
    types::{BorderStyle, Color},
    Client, Desktop, Ring, Workspace,
};
use crate::platform::x::{XConn, XWindow, XWindowID};

/// An object that can provide information about window manager
/// configuration at runtime.
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
/// needed type, by calling [`downcast_ref`][1] on it:
///
///
/// ```rust
/// use toaruwm::manager::RuntimeConfig;
///
/// fn try_getting_key(rtcfg: Box<dyn RuntimeConfig>) {
///     /* try to extract our item named "foo" from rtcfg */
///     let Some(val) = rtcfg.get_key("foo") else {
///         println!("no foo... T_T");
///         return
///     };     /* we're expecting 'foo' to be of type i32 */
///     if let Some(s) = val.downcast_ref::<i32>() {
///         println!("we got foo!");
///     } else {
///         println!("oh no, wrong type!");
///     }
/// }
/// ```
///
/// A provided method, `get_key_static`, does this call for you,
/// but the trade-off is that it cannot be called on a trait object.
///
/// See the [module-level documentation][2] on the [`Any`] trait for
/// more details.
///
/// [1]: https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref
/// [2]: std::any
pub trait RuntimeConfig {
    /// Return information about the floating classes.
    fn float_classes(&self) -> &[String];

    /// Return information about the window border thickness.
    fn border_px(&self) -> u32;

    /// The border color associated with a given [`BorderStyle`].
    fn border_style(&self, style: BorderStyle) -> Color;

    /// Return information about the gaps between windows.
    fn window_gap(&self) -> u32;

    /// Return whether the focus should follow the pointer.
    fn focus_follows_ptr(&self) -> bool;

    /// Retrieve arbitrary key value pairs from storage.
    ///
    /// Should return None if the key does not exist in
    /// storage.
    fn get_key(&self, key: &str) -> Option<&dyn Any>;

    /// A monomorphizable, easier-to-use version of `get_key`.
    ///
    /// Rust's restrictions on trait objects prevent `get_key`
    /// from returning generic types, thus it has to return
    /// a trait object (i.e. `&dyn Any`), and rely on the caller
    /// to call `downcast_ref` themselves to get the concrete
    /// type. This method does that call for you.
    ///
    /// Unfortunately, this means that this method cannot be
    /// called on a trait object.
    fn get_key_static<V: Any>(&self, key: &str) -> Option<&V>
    where
        Self: Sized,
    {
        self.get_key(key).and_then(|v| v.downcast_ref::<V>())
    }
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
    pub(crate) window_gap: u32,
    pub(crate) focus_follows_ptr: bool,
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
    fn border_style(&self, style: BorderStyle) -> Color {
        match style {
            BorderStyle::Focused => self.focused,
            BorderStyle::Unfocused => self.unfocused,
            BorderStyle::Urgent => self.urgent,
        }
    }

    fn window_gap(&self) -> u32 {
        self.window_gap
    }

    fn focus_follows_ptr(&self) -> bool {
        self.focus_follows_ptr
    }

    fn get_key(&self, key: &str) -> Option<&dyn Any> {
        self.keys.get(&key.to_string()).map(|v| v as &dyn Any)
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
    C: RuntimeConfig,
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
