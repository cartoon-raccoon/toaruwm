//! The window manager itself, and associated modules.

#![allow(unused_variables)] //fixme

/// Macros and storage types for window manager hooks.
pub mod hooks;
pub mod state;

mod manager;

#[doc(inline)]
pub use manager::Toaru;

#[doc(inline)]
pub use hooks::{Hook, Hooks};
#[doc(inline)]
pub use state::ToaruState;

/// Removes the focused window if under layout.
macro_rules! _rm_if_under_layout {
    ($_self:expr, $id:expr) => {
        let is_under_layout = $_self.desktop.current().has_window_in_layout($id);

        if is_under_layout {
            $_self.desktop.current_mut().remove_from_layout(
                &$_self.platform.handle(),
                $id,
                $_self.screens.focused().unwrap(),
                &$_self.config,
            );
        }
    };
}


