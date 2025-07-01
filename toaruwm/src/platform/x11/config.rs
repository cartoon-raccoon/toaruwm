//! X11-specific configuration.

use toaru_macro::ConfigSection;

use crate::config::ConfigSection;

/// A type that returns X11-specific configs.
pub trait X11Config: ConfigSection {
    
}

/// An implementation of [`X11Config`].
#[derive(Debug, Clone, Copy, PartialEq, ConfigSection)]
pub struct ToaruX11Config {}

impl X11Config for ToaruX11Config {}