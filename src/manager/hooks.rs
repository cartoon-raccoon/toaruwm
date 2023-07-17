use std::collections::HashMap;

use super::state::State;
use super::WindowManager;

/// Arbitrary code that can be run by the window manager.
///
/// A `Hook` is just a wrapper around a boxed trait object
/// implementing [`FnMut`].
/// You would generally use this through the [`hook`](crate::hook)
/// macro's much more ergonomic interface.
pub type Hook<X, C> = Box<dyn FnMut(&mut WindowManager<X, C>)>;

/// Macro for creating a hook that can be run by the window manager.
///
/// It accepts closure syntax, for example:
///
/// ## Example
/// ```ignore
/// # use toaruwm::WindowManager;
/// # //todo: implement dummy connection for this to work
/// use toaruwm::hook;
///
/// let dump_state = hook!(|wm| {
///     wm.dump_internal_state();
/// });
/// # //todo: insert this into a window manager for type inference
/// ```
///
/// And you can then insert this hook into a `Hooks`, which
/// is then passed into the `WindowManager` via `Config`.
///
#[macro_export]
macro_rules! hook {
    (|$wm:ident| $code:tt) => {
        Box::new(
            |$wm: &mut WindowManager<_,_>| $code
        ) as Box<dyn FnMut(&mut WindowManager<_,_>)>
    };
    (move |$wm:ident| $code:tt) => {
        Box::new(
            move |$wm: &mut WindowManager<_,_>| $code
        ) as Box<dyn FnMut(&mut WindowManager<_,_>)>
    };
}

/// Hooks that can be run by the window manager.
pub type Hooks<X, C> = HashMap<State, Vec<Hook<X, C>>>;
//todo: make this actually wrap the hashmap
