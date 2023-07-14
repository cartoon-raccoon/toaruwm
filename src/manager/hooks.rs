use std::collections::HashMap;

use super::WindowManager;
use super::state::State;

/// Arbitrary code that can be run by the window manager.
/// 
/// A `Hook` is just a wrapper around a boxed trait object
/// implementing [`FnMut`].
/// You would generally use this through the [`hook`] macro's
/// much more ergonomic interface.
pub type Hook<X> = Box<dyn FnMut(&mut WindowManager<X>)>;

/// Macro for creating a hook that can be run by the window manager.
/// 
/// It accepts closure syntax, for example:
/// 
/// ## Example
/// ```rust
/// use toaruwm::hook;
/// 
/// let dump_state = hook!(|wm| {
///     wm.dump_internal_state();
/// });
/// ```
/// 
/// And you can then insert this hook into a `Hooks`, which
/// is then passed into the `WindowManager` via `Config`.
/// 
#[macro_export]
macro_rules! hook {
    (|$wm:ident| $code:tt) => {
        Box::new(|$wm: &mut WindowManager<_>| $code)
        as Box<dyn FnMut(&mut WindowManager<_>)>
    };
    (move |$wm:ident| $code:tt) => {
        Box::new(move |$wm: &mut WindowManager<_>| $code)
        as Box<dyn FnMut(&mut WindowManager<_>)>
    }
}

/// Hooks that can be run by the window manager.
pub type Hooks<X> = HashMap<State, Vec<Hook<X>>>;
//todo: make this actually wrap the hashmap