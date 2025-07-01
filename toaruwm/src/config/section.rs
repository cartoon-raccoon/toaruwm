//! Configuration sections.

use std::any::Any;

/// An object that can act as a section in a Manager configuration.
/// 
/// This trait is the supertrait for any type that can serve as a
/// section in a Manager configuration. Each section that handles a specific
/// type of configuration (e.g. window decoration) uses this trait as a
/// supertrait, and the methods that define that trait's interface expose
/// methods that retrieve that specific key.
/// 
/// # Retrieving Arbitrary Values
///
/// One of `ConfigSection`'s required methods is `get_key`, which
/// returns a dynamically typed trait object (i.e. `&dyn Any`).
///
/// It is then up to the caller to see if this object is of the
/// needed type, by calling [`downcast_ref`][1] on it:
///
///
/// ```rust
/// use toaruwm::config::ConfigSection;
///
/// fn try_getting_key(rtcfg: Box<dyn ConfigSection>) {
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
pub trait ConfigSection {
    /// Retrieve arbitrary key value pairs from storage.
    ///
    /// Should return None if the key does not exist in storage.
    fn get_key(&self, key: &str) -> Option<&dyn Any>;

    /// Retrieve a subsection for this section.
    /// 
    /// Should return None if the subsection does not exist.
    fn subsection(&self, name: &str) -> Option<&dyn ConfigSection>;

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