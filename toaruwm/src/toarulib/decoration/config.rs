//! Config section implementation for window decorations.

use crate::config::ConfigSection;
use crate::types::Color;

use super::BorderStyle;

/// A type that can supply server-wide decoration configuration.
pub trait DecorationConfig: ConfigSection {

    /// The border thickness.
    fn border_px(&self) -> u8;

    /// The border colors associated with a given [`BorderStyle`].
    fn border_style(&self, style: BorderStyle) -> &[Color];
}