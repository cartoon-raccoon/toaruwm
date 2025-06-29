#![allow(missing_docs)] // fixme: remove when we get a working prototype

pub mod state;
pub mod backend;
pub mod util;
pub mod render;
pub mod window;
pub mod output;

pub(crate) mod handlers;

pub(self) mod convert;

pub use state::WlState;

use super::{ClientData, Platform, PlatformType};

mod wayland;
#[doc(inline)]
pub use wayland::*;

pub(self) mod prelude {
    pub use super::Wayland;
    pub use crate::config::RuntimeConfig;
    pub use super::backend::WaylandBackend;
}