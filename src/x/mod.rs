//! This module provides ToaruWM's main interface to the X server.
//! The core of this module is the `XConn` trait, which defines the
//! interface by which the window manager retrives data from and
//! sets data on the X server, using ToaruWM types for abstraction.
//!
//! For concrete implementation of the traits exported here, this module
//! offers two submodules which each contain implementations using the XCB
//! and X11RB backing libraries respectively.

/// Types for dealing with atoms and their string representations.
pub mod atom;
/// Core types and traits for interfacing with the X server.
pub mod core;
/// Convenience types for cursors.
pub mod cursor;
/// Types for representing X events.
pub mod event;
/// Types for working with keyboard and mouse input.
pub mod input;
/// Types for working with window properties.
pub mod property;

/// Implementation of `XConn` backed by the `x11rb` library.
pub mod x11rb;
/// Implementation of `XConn` backed by the `xcb` library.
pub mod xcb;

pub use self::core::{XAtom, XConn, XError, XWindow, XWindowID};
pub use atom::{Atom, Atoms};
pub use event::XEvent;
pub use property::*;

pub use self::x11rb::X11RBConn;
pub use self::xcb::XCBConn;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) mod dummy;

// various backend-agnostic conversion implementations

use std::string::FromUtf8Error;

impl From<FromUtf8Error> for XError {
    fn from(e: FromUtf8Error) -> XError {
        XError::InvalidPropertyData(format!("Invalid UTF8 data: {}", e))
    }
}

use std::array::TryFromSliceError as TFSError;

impl From<TFSError> for XError {
    fn from(_: TFSError) -> Self {
        XError::ConversionError
    }
}

use std::convert::TryFrom;

use crate::keybinds::ButtonIndex;
use crate::x::core::Result;

impl TryFrom<u8> for ButtonIndex {
    type Error = XError;

    fn try_from(from: u8) -> Result<ButtonIndex> {
        match from {
            1 => Ok(ButtonIndex::Left),
            2 => Ok(ButtonIndex::Middle),
            3 => Ok(ButtonIndex::Right),
            4 => Ok(ButtonIndex::Button4),
            5 => Ok(ButtonIndex::Button5),
            _ => Err(XError::ConversionError),
        }
    }
}
