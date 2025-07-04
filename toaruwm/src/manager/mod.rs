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

use crate::platform::Platform;
use crate::config::RuntimeConfig;
use crate::types::{Rectangle, Logical};

/// A type that implements window management functionality.
pub trait Manager {
    /// The platform used by the Manager.
    type Platform: Platform;

    /// The runtime configuration of the Manager.
    type Config: RuntimeConfig;

    /// Add an output to the Manager..
    fn add_output(&mut self, output: <Self::Platform as Platform>::Output);

    /// Remove an output from the Manager.
    fn remove_output(&mut self, output: &<Self::Platform as Platform>::Output);

    /// Insert a new window into the Manager.
    fn insert_window(&mut self, id: <Self::Platform as Platform>::WindowId) -> Option<Rectangle<i32, Logical>>;
}