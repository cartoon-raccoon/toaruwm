//! Runtime configuration for a `Toaru` instance.

use std::any::Any;
use std::fmt::Debug;

use crate::{core::{
    types::{BorderStyle, Color},
}};

use crate::config::{OutputLayout, section::{WaylandConfig, X11Config}};

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
/// [3]: RuntimeConfig::wayland_cfg
/// [4]: RuntimeConfig::x11_cfg
pub trait RuntimeConfig: Debug {
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

    /// Return the outputs and their layout.
    fn outputs(&mut self) -> &mut OutputLayout;

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

    /// Return Wayland-specific configuration options.
    /// 
    /// Re-implement this if you want to make your RuntimeConfig object
    /// compatible with Wayland.
    fn wayland_cfg(&self) -> Option<Box<dyn WaylandConfig>> {
        None
    }

    /// Return X11-specific configuration options.
    /// 
    /// Re-implement this if you want to make your RuntimeConfig object
    /// compatible with X11.
    fn x11_cfg(&self) -> Option<Box<dyn X11Config>> {
        None
    }
}