//! Config section implementation for window decorations.

use toaru_macro::ConfigSection;

use crate::config::ConfigSection;
use crate::types::Gradient;

use super::BorderStyle;

/// A type that can supply server-wide decoration configuration.
pub trait DecorationConfig: ConfigSection {

    /// The border thickness.
    fn border_px(&self) -> u8;

    /// The border colors associated with a given [`BorderStyle`].
    fn border_style(&self, style: BorderStyle) -> &Gradient;
}

/// An implementation of [`DecorationConfig`].
#[derive(Debug, Clone, ConfigSection)]
pub struct ToaruDecorationConfig {
    #[key]
    border_px: u8,
    #[key]
    border_focused: Gradient,
    #[key]
    border_unfocused: Gradient,
    #[key]
    border_urgent: Gradient,
}

impl DecorationConfig for ToaruDecorationConfig {
    fn border_px(&self) -> u8 {
        self.border_px
    }

    fn border_style(&self, style: BorderStyle) -> &Gradient {
        match style {
            BorderStyle::Focused => &self.border_focused,
            BorderStyle::Unfocused => &self.border_unfocused,
            BorderStyle::Urgent => &self.border_urgent,
        }
    }
}