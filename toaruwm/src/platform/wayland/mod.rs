#![allow(missing_docs)] // fixme: remove when we get a working prototype

pub mod state;
pub mod backend;
pub mod util;
pub mod render;
pub mod window;
pub mod input;

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

use crate::types::{Point, Logical, Transform};
use crate::config::{OutputScale, OutputMode};
use crate::platform::PlatformOutput;

/// An Output as used by the Wayland platform.
pub type WaylandOutput = smithay::output::Output;

impl PlatformOutput for WaylandOutput {
    fn name(&self) -> String {
        self.name()
    }

    fn location(&self) -> Point<i32, Logical> {
        self.current_location().into()
    }
    
    fn transform(&self) -> Transform {
        self.current_transform().into()
    }

    fn scale(&self) -> OutputScale {
        self.current_scale().into()
    }

    fn current_mode(&self) -> Option<OutputMode> {
        self.current_mode().map(|mode| mode.into())
    }

    fn preferred_mode(&self) -> Option<OutputMode> {
        self.preferred_mode().map(|mode| mode.into())
    }

    fn modes(&self) -> Vec<OutputMode> {
        self.modes()
            .into_iter()
            .map(|mode| mode.into())
            .collect()
    }
}