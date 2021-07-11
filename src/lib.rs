#[macro_use]
extern crate bitflags;

#[macro_use]
mod log;

pub mod x;
pub mod core;
pub mod layouts;
pub mod manager;
pub mod keybinds;

pub(crate) mod util;

pub use crate::core::types;
pub use crate::x::core::Result as XResult;
pub use crate::x::core::XConn;
pub use crate::x::xcb::XCBConn;
pub use crate::manager::WindowManager;

use std::ops::FnMut;
use std::io;
use std::num::ParseIntError;

/// Convenience function for creating a XCB-backed WindowManager.
pub fn xcb_backed_wm() -> XResult<WindowManager<XCBConn>> {
    let mut xcbconn = XCBConn::connect()?;
    xcbconn.init()?;

    let wm = WindowManager::new(xcbconn);

    Ok(wm)
}

use thiserror::Error;
use crate::x::core::{
    XError,
    XWindowID,
};

/// Everything that could possibly go wrong while ToaruWM is running.
#[derive(Debug, Error, Clone)]
pub enum ToaruError {

    /// An error with the underlying X connection.
    #[error(transparent)]
    XConnError(XError),

    /// Unable to spawn process.
    #[error("Error while running program: {0}")]
    SpawnProc(String),

    /// Unable to parse an X data type into a type known to ToaruWM.
    #[error("Could not parse X data type from integer")]
    ParseInt,

    #[error("Could not parse keybind \"{0}\"")]
    ParseKeybind(String),

    /// Unable to convert external data into an internal Toaru datatype.
    #[error("Could not convert external data type for internal use")]
    ConversionError,

    /// Received a reference to a client not tracked by ToaruWM.
    #[error("Unknown client {0}")]
    UnknownClient(XWindowID),

    /// An request to switch to a workspace unknown to ToaruWM.
    #[error("Unknown workspace {0}")]
    UnknownWorkspace(String),

    /// An invalid point on the root window.
    #[error("Invalid point ({0}, {1})")]
    InvalidPoint(i32, i32),

    /// An error not covered by ToaruWM.
    #[error("Error: {0}")]
    OtherError(String),
}

impl From<XError> for ToaruError {
    fn from(e: XError) -> ToaruError {
        ToaruError::XConnError(e)
    }
}

impl From<io::Error> for ToaruError {
    fn from(e: io::Error) -> ToaruError {
        ToaruError::SpawnProc(e.to_string())
    }
}

impl From<ParseIntError> for ToaruError {
    fn from(_: ParseIntError) -> ToaruError {
        ToaruError::ParseInt
    }
}

/// The general result type used by ToaruWM.
pub type Result<T> = ::core::result::Result<T, ToaruError>;

/// An error handler that can be used to handle an error type.
/// 
/// Typically this would be a standard logging function that writes
/// to a file or stdout, but it can be anything.
pub type ErrorHandler = Box<dyn FnMut(ToaruError)>;


