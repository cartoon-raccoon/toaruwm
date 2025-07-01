//! Configuration sections.

use std::any::Any;

/// An object that can act as a section in a Manager configuration.
pub trait ConfigSection {
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

/// A type that returns Wayland-specific configs.
pub trait WaylandConfig {

}

/// A type that returns X11-specific configs.
pub trait X11Config {

}