//! Types and traits providing a unified interface with the X server.
//! 
//! This module provides ToaruWM's main interface to the X server.
//! The core of this module is the `XConn` trait, which defines the
//! interface by which the window manager retrives data from and
//! sets data on the X server, using ToaruWM types for abstraction.
//!
//! For concrete implementation of the traits exported here, this module
//! offers two submodules which each contain implementations using the XCB
//! and X11RB backing libraries respectively.

pub mod atom;
pub mod core;
pub mod cursor;
pub mod event;
pub mod input;
pub mod property;

/// Implementation of `XConn` backed bt the `x11rb` library.
pub mod x11rb;
/// Implementation of `XConn` backed by the `xcb` library.
pub mod xcb;

#[doc(inline)]
pub use self::core::{XAtom, XConn, XError, XWindow, XWindowID};
#[doc(inline)]
pub use atom::{Atom, Atoms};
#[doc(inline)]
pub use event::XEvent;
pub(crate) use property::*;

#[doc(inline)]
pub use self::x11rb::X11RBConn;
#[doc(inline)]
pub use self::xcb::XCBConn;


/* since xconn implementations can only be tested
on a system with an X server running, disable this
unless we specifically enable the `protocol` cfg flag
which should only be be used if testing locally */
#[cfg(all(test, protocol))]
mod tests;

/* since the dummy connection is used for testing
higher-level code and does not actually interact with
an actual X server, keep this enabled for standard testing */
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
