#![allow(missing_docs)] // fixme: remove when we get a working prototype

pub mod state;
pub mod handlers;
pub mod backend;
pub mod util;
pub mod render;

mod wayland;

pub(self) mod convert;

pub use state::WlState;

use super::{ClientData, Platform, PlatformType};

#[doc(inline)]
pub use wayland::*;