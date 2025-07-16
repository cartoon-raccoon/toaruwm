#![allow(missing_docs)] // fixme: remove when we get a working prototype

pub mod backend;
pub mod util;
pub mod render;
pub mod window;
pub mod input;
pub mod config;

pub(crate) mod handlers;

pub(self) mod convert;

pub(crate) mod macros;

mod wayland;

// Public re-exports.
#[doc(inline)]
pub use wayland::*;
#[doc(inline)]
pub use window::WaylandWindow;
#[doc(inline)]
pub use handlers::WaylandState;
pub use config::{WaylandConfig, ToaruWaylandConfig};

pub(self) mod prelude {
    //! Convenient all-import for all traits that Wayland has requirements for.
    //pub use super::WaylandImpl;
    pub use super::Wayland;
    pub use crate::config::RuntimeConfig;
    pub use crate::Manager;
    pub use super::backend::WaylandBackend;
}

use crate::types::{Point, Logical, Transform};
use crate::config::{OutputScale, OutputMode, OutputInfo};
use crate::types::{Rectangle, Size, Scale};

use smithay::output::PhysicalProperties;

/// An Output as used by the Wayland platform.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WaylandOutput(smithay::output::Output);

impl WaylandOutput {
    pub fn name(&self) -> String {
        self.0.name()
    }

    pub fn info(&self) -> OutputInfo {
        let PhysicalProperties { make, model, .. } = self.0.physical_properties();
        OutputInfo {
            make: Some(make),
            model: Some(model)
        }
    }

    pub fn location(&self) -> Point<i32, Logical> {
        self.0.current_location().into()
    }
    
    pub fn transform(&self) -> Transform {
        self.0.current_transform().into()
    }

    pub fn scale(&self) -> OutputScale {
        self.0.current_scale().into()
    }

    pub fn current_mode(&self) -> Option<OutputMode> {
        self.0.current_mode().map(|mode| mode.into())
    }

    pub fn preferred_mode(&self) -> Option<OutputMode> {
        self.0.preferred_mode().map(|mode| mode.into())
    }

    pub fn modes(&self) -> Vec<OutputMode> {
        self.0.modes()
            .into_iter()
            .map(|mode| mode.into())
            .collect()
    }

    /// Returns the Geometry of the Output, if its mode is set.
    pub fn geometry(&self) -> Option<Rectangle<i32, Logical>> {
        let Some(mode) = self.current_mode() else {
            return None
        };

        let size = match self.scale() {
            OutputScale::Fractional(f) | OutputScale::Split {fractional: f, ..} => {
                let Size {width, height, .. } = mode.size.as_f64().as_logical(Scale::uniform(f));
                Size::<i32, Logical>::new(width as i32, height as i32)
            }
            OutputScale::Integer(i) => {
                mode.size.as_logical(Scale::uniform(i))
            }
        };

        Some(Rectangle {point: self.location(), size})
    }
}