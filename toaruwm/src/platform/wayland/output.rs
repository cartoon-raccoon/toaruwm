//! A Wayland representation of an Output.

use std::sync::{Arc, RwLock, Weak};

use smithay::output::Output;

use crate::types::{Point, Logical, Transform};
use crate::config::{OutputScale, OutputMode};
use crate::platform::PlatformOutput;

/// An output, as represented by Wayland.
#[derive(Debug)]
pub struct WaylandOutput {
    inner: Arc<RwLock<Output>>,
}

impl WaylandOutput {
    /// Creates a new WaylandOutput from a given output.
    pub fn new(output: Output) -> Self {
        Self {
            inner: Arc::new(RwLock::new(output))
        }
    }

    pub fn downgrade(&self) -> WeakWaylandOutput {
        WeakWaylandOutput {
            inner: Arc::downgrade(&self.inner)
        }
    }
}

impl Clone for WaylandOutput {
    fn clone(&self) -> WaylandOutput {
        WaylandOutput {
            inner: Arc::clone(&self.inner)
        }
    }
}

/// A weakly ref-counted reference to a WaylandOutput.
#[derive(Debug)]
pub struct WeakWaylandOutput {
    inner: Weak<RwLock<Output>>
}

impl WeakWaylandOutput {
    /// Try to upgrade self back into a strong reference.
    /// 
    /// Returns None if the underlying WaylandOutput was dropped.
    pub fn upgrade(&self) -> Option<WaylandOutput> {
        Weak::upgrade(&self.inner).map(|inner| WaylandOutput {inner})
    }
}

impl PlatformOutput for WaylandOutput {
    fn name(&self) -> String {
        self.inner.read().unwrap().name()
    }

    fn location(&self) -> Point<i32, Logical> {
        self.inner.read().unwrap().current_location().into()
    }
    
    fn transform(&self) -> Transform {
        self.inner.read().unwrap().current_transform().into()
    }

    fn scale(&self) -> OutputScale {
        self.inner.read().unwrap().current_scale().into()
    }

    fn current_mode(&self) -> Option<OutputMode> {
        self.inner.read().unwrap().current_mode().map(|mode| mode.into())
    }

    fn preferred_mode(&self) -> Option<OutputMode> {
        self.inner.read().unwrap().preferred_mode().map(|mode| mode.into())
    }

    fn modes(&self) -> Vec<OutputMode> {
        self.inner.read().unwrap()
            .modes()
            .into_iter()
            .map(|mode| mode.into())
            .collect()
    }
}