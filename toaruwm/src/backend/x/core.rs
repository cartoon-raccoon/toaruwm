//! Core types and traits for interfacing with the X server.
//!
//! Core functionality of ToaruWM's interface with the X server.
//!
//! This module defines core types and traits used throughout this
//! crate for directly interacting with the X server.

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Deref, DerefMut, Not};
use std::fmt::{self, Display};

use thiserror::Error;
use tracing::{debug, error};


use super::{
    atom::Atom,
    input::KeyButMask,
};
use crate::types::Geometry;
use crate::backend::BackendError;

#[doc(inline)]
pub use super::traits::{XCore, XConn, RandR, Xkb};

//* ========== X WINDOW THINGS ========== *//

/// General constant for expressing None when passing X IDs.
pub const XID_NONE: Xid = Xid::zero();

/// Wrapper type to represent IDs used by the X server.
///
/// This is used by the server to identify all sorts
/// of X window resources, including windows and atoms.
///
/// You can create an Xid from a `u32`:
///
/// ```rust
/// use toaruwm::x::Xid;
///
/// let id = Xid::from(69);
/// let val = id.val();
///
/// assert_eq!(val, 69);
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub struct Xid(pub(crate) u32);

impl Xid {
    /// Return an Xid set to 0.
    pub const fn zero() -> Self {
        Xid(0)
    }

    /// Returns the internal value of the Xid.
    pub const fn val(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Xid {
    fn from(f: u32) -> Xid {
        Xid(f)
    }
}

impl Display for Xid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Xid({})", self.0)
    }
}

impl Deref for Xid {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.0
    }
}

impl DerefMut for Xid {
    fn deref_mut(&mut self) -> &mut u32 {
        &mut self.0
    }
}

/// An X server ID for a given window.
pub type XWindowID = Xid;

/// An X Atom.
pub type XAtom = Xid;

/// A marker trait to signal that a type can be treated as a bitmask.
///
/// This means that the type supports bitmask operations such as
/// bitwise AND, bitwise OR, bitwise NOT, etc.
pub trait BitMask
where
    Self: BitAnd + BitOr + Not + BitAndAssign + BitOrAssign + Sized,
{
}

// Blanket implementation for Bitmask
impl<T> BitMask for T where T: BitAnd + BitOr + Not + BitAndAssign + BitOrAssign + Sized {}

/// Window stacking modes defined by the X Protocol.
///
/// Each variant may carry a `sibling` window ID, that
/// changes the semantics of the `StackMode`.
///
/// The exact semantics of this difference are explained
/// [here](https://tronche.com/gui/x/xlib/window/configure.html).
#[derive(Clone, Copy, Debug)]
pub enum StackMode {
    /// Stack the window at the top of the stack.
    ///
    /// If a sibling is specified, the window is instead stacked
    /// just above the specified sibling.
    Above(Option<XWindowID>),
    /// Stack the window at the bottom of the stack.
    ///
    /// If a sibling is specified, the window is instead stacked
    /// just below the specified sibling.
    Below(Option<XWindowID>),
    /// If any sibling occludes the window, the window
    /// is stacked at the top of the stack.
    ///
    /// If a sibling is specified, then the window is
    /// stacked only if the sibling occludes it.
    TopIf(Option<XWindowID>),
    /// If the window occludes any sibling, the window
    /// is stacked at the bottom of the stack.
    ///
    /// If a sibling is specified, then the window is
    /// stacked only if it occludes any sibling.
    BottomIf(Option<XWindowID>),
    /// TODO
    Opposite(Option<XWindowID>),
}

/// Reply to a pointer query.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PointerQueryReply {
    /// If the pointer is on the current screen.
    pub same_screen: bool,
    /// The root window.
    pub root: XWindowID,
    /// The child containing the pointer.
    pub child: XWindowID,
    /// The x-coordinate of the pointer relative to the root window.
    pub root_x: i32,
    /// The y-coordinate of the pointer relative to the root window.
    pub root_y: i32,
    /// The x-coordinate of the pointer relative to the child window.
    pub win_x: i32,
    /// The y-coordinate of the pointer relative to the child window.
    pub win_y: i32,
    /// The logical state of the buttons and modkeys.
    pub mask: KeyButMask,
}

/// Representation of an X window with additional data (geometry).
#[derive(Debug, Clone, Copy)]
pub struct XWindow {
    /// The X ID assigned to the window.
    pub id: XWindowID,
    /// The geometry of the window as stored on the X server.
    pub geom: Geometry,
}

impl XWindow {
    /// Creates an `XWindow` with all fields zeroed.
    pub fn zeroed() -> Self {
        XWindow {
            id: Xid(0),
            geom: Geometry::zeroed(),
        }
    }

    /// Creates an `XWindow` with the given data.
    pub fn with_data(id: XWindowID, geom: Geometry) -> Self {
        XWindow { id, geom }
    }
}

impl PartialEq for XWindow {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<XWindowID> for XWindow {
    fn from(from: XWindowID) -> Self {
        Self {
            id: from,
            geom: Geometry {
                x: 0,
                y: 0,
                height: 0,
                width: 0,
            },
        }
    }
}

/// The type of window that you want to create.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowClass {
    /// An invisible window to make API calls
    CheckWin,
    /// A window that only accepts input
    InputOnly,
    /// A regular window. The Atom provided should be a valid
    /// _NET_WM_WINDOW_TYPE, and the u32 is the border width
    /// in pixels.
    ///
    /// This should be the value returned by a
    /// [`RuntimeConfig`](crate::manager::RuntimeConfig)'s
    /// `border_px` method.
    InputOutput(Atom, u32),
    //todo: add additional variants for new window types?
}

impl XWindow {
    /// Sets the geometry using an XConn object.
    ///
    /// This method requests the X server directly for the
    /// geometry of the window and updates its internal fields
    /// accordingly.
    pub fn set_geometry_conn<X: XConn>(&mut self, conn: &X) {
        match conn.get_geometry(self.id) {
            Ok(geom) => {
                debug!(
                    "Updating geometry (conn): x: {}, y: {}, h: {}, w: {}",
                    geom.x, geom.y, geom.height, geom.width
                );
                self.geom = geom;
            }

            Err(e) => {
                error!("{}", e);
            }
        }
    }

    /// Sets the geometry using a provided Geometry.
    ///
    /// Note that this does not update the geometry as tracked by
    /// the X server, and so a request should be made to the server
    /// to update the geometry there as well.
    pub fn set_geometry(&mut self, geom: Geometry) {
        debug!(
            "Updating geometry for window {}: x: {}, y: {}, h: {}, w: {}",
            self.id, geom.x, geom.y, geom.height, geom.width
        );
        self.geom = geom;
    }

    /// Sets the width of the window.
    pub fn set_width(&mut self, x: i32) {
        self.geom.width = x;
    }

    /// Sets the height of the window.
    pub fn set_height(&mut self, y: i32) {
        self.geom.height = y;
    }

    /// Sets the x coordinate of the window.
    pub fn set_pos_x(&mut self, x: i32) {
        self.geom.x = x;
    }

    /// Sets the y coordinate of the window.
    pub fn set_pos_y(&mut self, y: i32) {
        self.geom.y = y;
    }

    /// Updates the width by a given delta.
    pub fn update_width(&mut self, dx: i32) {
        self.geom.width += dx;
    }
    /// Updates the height by given delta.
    pub fn update_height(&mut self, dy: i32) {
        self.geom.height += dy;
    }
    /// Updates the x coordinate of the window by a given delta.
    pub fn update_pos_x(&mut self, dx: i32) {
        self.geom.x += dx;
    }
    /// Updates the y coordinate of the window by a given delta.
    pub fn update_pos_y(&mut self, dy: i32) {
        self.geom.y += dy;
    }
}

/// Possible errors returned by the X connection.
#[non_exhaustive]
#[derive(Debug, Error, Clone)]
pub enum XError {
    /// An error when establishing a connection with the server.
    #[error("X connection error: {0}")]
    Connection(String),

    /// An error caused by a malformed protocol request.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// No screens were found by the window manager.
    #[error("Could not find screens from X server")]
    NoScreens,

    /// An invalid screen was selected.
    #[error("Unknown screen selected")]
    InvalidScreen,

    /// Some error caused by a RandR request or event.
    #[error("RandR error: {0}")]
    RandrError(RandrErrorKind),

    /// Some error caused by an XKB request or event.
    #[error("XKB error: {0}")]
    XKBError(XKBErrorKind),

    /// An internal server error.
    #[error("X server error: {0}")]
    ServerError(String),

    /// An error converting property or client message data.
    #[error("Error converting client message data")]
    ConversionError,

    /// A request for window properties returned malformed data.
    #[error("Invalid property data: {0}")]
    InvalidPropertyData(String),

    /// The request could not be fulfilled by the X server.
    #[error("Could not complete specified request: {0}")]
    RequestError(&'static str),

    /// Some error not tracked by ToaruWM.
    #[error("{0}")]
    OtherError(String),
}

impl From<XError> for BackendError {
    fn from(from: XError) -> BackendError {
        BackendError::XError(from)
    } 
}

/// The kind of error generated by RandR.
#[non_exhaustive]
#[derive(Debug, Clone, Error)]
pub enum RandrErrorKind {
    /// An incompatible version of RandR is present on the server.
    #[error("incompatible randr version, got {0}.{1}, requires 1.4")]
    IncompatibleVer(u32, u32),

    /// Randr could not be found on the X server.
    #[error("RandR is not present on this installation of Xorg")]
    NotPresent,

    /// An unspecified RandR error.
    #[error("{0}")]
    Other(String),
}

/// The kind of error generated by XKB.
#[non_exhaustive]
#[derive(Debug, Clone, Error)]
pub enum XKBErrorKind {
    /// An incompatible version of XKB is present on the server.
    #[error("incompatible XKB version, got {0}.{1}")]
    IncompatibleVer(u16, u16),

    /// XKB could not be found on the X server.
    #[error("XKB is not present on this installation of Xorg")]
    NotPresent,

    /// An unspecified XKB error.
    #[error("{0}")]
    Other(String),
}

/// Result type for XConn.
pub type Result<T> = ::core::result::Result<T, XError>;


