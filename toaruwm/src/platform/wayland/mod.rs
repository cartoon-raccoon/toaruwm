#![allow(missing_docs)] // fixme: remove when we get a working prototype

pub mod backend;
pub mod util;
pub mod render;
pub mod window;
pub mod input;
pub mod config;

pub(crate) mod handlers;

pub(self) mod convert;

mod wayland;

// Public re-exports.
#[doc(inline)]
pub use wayland::*;
#[doc(inline)]
pub use handlers::WaylandState;
pub use config::{WaylandConfig, ToaruWaylandConfig};

pub(self) mod prelude {
    //! Convenient all-import for all traits that Wayland has requirements for.
    //pub use super::WaylandImpl;
    pub use super::Wayland;
    pub use crate::config::RuntimeConfig;
    pub use super::backend::WaylandBackend;
}

use crate::types::{Point, Logical, Transform};
use crate::config::{OutputScale, OutputMode, OutputInfo};
use crate::platform::PlatformOutput;
use super::{ClientData, Platform, PlatformType};

use smithay::output::PhysicalProperties;

/// An Output as used by the Wayland platform.
pub type WaylandOutput = smithay::output::Output;

impl PlatformOutput for WaylandOutput {
    fn name(&self) -> String {
        self.name()
    }

    fn info(&self) -> OutputInfo {
        let PhysicalProperties { make, model, .. } = self.physical_properties();
        OutputInfo {
            make: Some(make),
            model: Some(model)
        }
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