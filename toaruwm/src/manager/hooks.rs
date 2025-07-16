use std::collections::HashMap;

use super::state::State;
use super::Toaru;

/// Arbitrary code that can be run by the window manager.
///
/// A `Hook` is just a wrapper around a boxed trait object
/// implementing [`FnMut`].
/// You would generally use this through the [`hook`](crate::hook)
/// macro's much more ergonomic interface.
pub type Hook<C> = Box<dyn for<'t> FnMut(&'t mut Toaru<C>)>;

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
        Box::new(|$wm: &mut Toaru<_, _>| $code) as Box<dyn FnMut(&mut Toaru<_, _>)>
    };
    (move |$wm:ident| $code:tt) => {
        Box::new(move |$wm: &mut Toaru<_, _>| $code)
            as Box<dyn FnMut(&mut Toaru<_, _>)>
    };
}

/// Hooks that can be run by the window manager.
pub type Hooks<C> = HashMap<State, Vec<Hook<C>>>;
//todo: make this actually wrap the hashmap
