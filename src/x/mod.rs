//! This module provides ToaruWM's main interface to the X server.
//! The core of this module is the `XConn` trait, which defines the
//! interface by which the window manager retrives data from and 
//! sets data on the X server, using ToaruWM types for abstraction.
//! 
//! For concrete implementation of the traits exported here, this module
//! offers two submodules which each contain implementations using the XCB
//! and X11RB backing libraries respectively.

pub mod core;
pub mod event;
pub mod atom;
pub mod property;
pub mod cursor;
pub mod input;

pub mod xcb;
pub mod x11rb;

pub use self::core::{
    XConn, XWindow, XWindowID, XAtom, XError
};
pub use event::XEvent;
pub use atom::{Atom, Atoms};
pub use property::*;

#[cfg(test)]
mod tests;

// various backend-agnostic conversion implementations

use std::string::FromUtf8Error;

impl From<FromUtf8Error> for XError {
    fn from(e: FromUtf8Error) -> XError {
        XError::InvalidPropertyData(
            format!("Invalid UTF8 data: {}", e)
        )
    }
}

use std::array::TryFromSliceError as TFSError;

impl From<TFSError> for XError {
    fn from(_: TFSError) -> Self {
        XError::ConversionError
    }
}

use std::convert::TryFrom;

use crate::x::core::Result;
use crate::keybinds::ButtonIndex;

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