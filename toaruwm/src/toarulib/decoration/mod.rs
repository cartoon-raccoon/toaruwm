//! Server-side window decorations.

pub mod config;

pub use config::DecorationConfig;

use crate::types::Color;

/// Window decorations for a single window.
#[derive(Debug, Clone)]
pub struct WindowDecoration {

}


pub struct Border {

}

/// Determines the colour that should be applied to
/// the window border.
///
/// The actual colour values are specified in `Config`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderStyle {
    /// The colour to be applied to the focused window.
    Focused,
    /// The colour to be applied to an unfocused window.
    Unfocused,
    /// The colour to applied when a window is marked as urgent.
    Urgent,
}