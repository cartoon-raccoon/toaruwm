//! Wayland-specific configuration.

use toaru_macro::ConfigSection;

use crate::config::ConfigSection;

/// A type that can provide Wayland-specific configuration at runtime.
pub trait WaylandConfig: ConfigSection {
    /// Whether direct scanout is enabled.
    fn direct_scanout(&self) -> bool;

    /// Whether variable refresh rate should be used where possible.
    fn vrr(&self) -> bool;
}

/// An implementation of [`WaylandConfig`].
#[derive(Debug, Clone, Copy, PartialEq, ConfigSection)]
pub struct ToaruWaylandConfig {
    #[key]
    direct_scanout: bool,
    
    #[key]
    vrr: bool,
}

impl Default for ToaruWaylandConfig {
    fn default() -> Self {
        Self {
            direct_scanout: false,
            vrr: true,
        }
    }
}

impl WaylandConfig for ToaruWaylandConfig {
    fn direct_scanout(&self) -> bool {
        self.direct_scanout
    }

    fn vrr(&self) -> bool {
        self.vrr
    }
}