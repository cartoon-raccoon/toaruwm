//! Types for working with X events.

use strum::{EnumIs};

use super::{
    core::{StackMode, XAtom},
    input::{KeyCode, ModMask},
    XWindowID,
};
use crate::bindings::Mousebind;
use crate::types::{Rectangle, Point, Logical, Physical};

/// Low-level wrapper around actual X server events.
///
/// Translated to EventActions by `WindowManager`.
#[derive(Debug, Clone, EnumIs)]
pub enum XEvent {
    /// Notification that a client has changed its configuration.
    ConfigureNotify(ConfigureEvent),
    /// Request for configuration from a client.
    ConfigureRequest(ConfigureRequestData),
    /// A Client is requesting to be mapped.
    MapRequest(XWindowID, bool), // bool: override_redirect
    /// A Client has mapped a window.
    MapNotify(XWindowID, bool), // bool: from root
    /// A Client has unmapped a window.
    UnmapNotify(XWindowID, bool), // bool: from root
    /// A Client has destroyed a window.
    DestroyNotify(XWindowID),
    /// The pointer has entered a window.
    ///
    /// The bool is whether the pointer is grabbed.
    EnterNotify(PointerEvent, bool),
    /// The pointer has left a window.
    ///
    /// The bool is whether the pointer is grabbed.
    LeaveNotify(PointerEvent, bool),
    /// A window was reparented.
    ReparentNotify(ReparentEvent),
    /// A window property was changed.
    PropertyNotify(PropertyEvent),
    /// A key combination was pressed.
    KeyPress(XWindowID, KeypressEvent),
    /// A key combination was released.
    //? does this need any more information?
    KeyRelease,
    /// A mouse button was pressed.
    MouseEvent(MouseEvent),
    /// A client message was received.
    ClientMessage(ClientMessageEvent),
    /// Received a randr notification.
    RandrNotify,
    /// Received a randr screen change notify event.
    ScreenChange,
    /// Unknown event type, used as a catchall for events not tracked by toaruwm.
    Unknown(String),
}

/// Data associated with a configure event.
#[derive(Debug, Clone, Copy)]
pub struct ConfigureEvent {
    /// Whether the window manager sent this event.
    pub from_root: bool,
    /// The window associated with the event.
    pub id: XWindowID,
    /// The new geometry requested by the window.
    pub geom: Rectangle<i32, Logical>,
    /// Is the window the root window
    pub is_root: bool,
}

/// Data associated with a configure request.
#[derive(Debug, Clone, Copy)]
pub struct ConfigureRequestData {
    /// The window associated with the event.
    pub id: XWindowID,
    /// The parent window of id.
    pub parent: XWindowID,
    /// Sibling window of id. Used if stack_mode is set.
    pub sibling: Option<XWindowID>,
    /// X coordinate to configure to.
    pub x: Option<i32>,
    /// Y coordinate to configure to.
    pub y: Option<i32>,
    /// Window height to configure to.
    pub height: Option<u32>,
    /// Window width to configure to.
    pub width: Option<u32>,
    /// Stack mode to configure to.
    pub stack_mode: Option<StackMode>,
    /// If the window to configure is root.
    pub is_root: bool,
}

/// Data associated with a reparent event.
#[derive(Debug, Clone, Copy)]
pub struct ReparentEvent {
    /// Whether the window manager sent this event.
    pub from_root: bool,
    /// The parent window.
    pub parent: XWindowID,
    /// The child of the parent window.
    pub child: XWindowID,
    /// Whether the child window is override-redirect.
    pub over_red: bool,
}

/// Data associated with a pointer change event (Enter, Leave).
#[derive(Debug, Clone, Copy)]
pub struct PointerEvent {
    /// The id of the event window.
    pub id: XWindowID,
    /// The absolute position of the pointer (relative to root).
    pub abs: Point<i32, Physical>,
    /// The relative position of the pointer to the event window.
    pub rel: Point<i32, Physical>,
}

/// Data associated with a property change event.
#[derive(Debug, Clone, Copy)]
pub struct PropertyEvent {
    /// The window associated with the event.
    pub id: XWindowID,
    /// The atom representing the change.
    pub atom: XAtom,
    /// The time of event.
    pub time: u32,
    /// Whether the property was deleted.
    pub deleted: bool,
}

/// Data associated with a key press event.
#[derive(Debug, Clone, Copy)]
pub struct KeypressEvent {
    /// The state of modifier keys was active at the time.
    pub mask: ModMask,
    /// The keycode of the key pressed.
    pub keycode: KeyCode,
}

/// Data associated with a button press event.
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    /// The window the pointer was on when the button was pressed.
    pub id: XWindowID,
    /// The location of the pointer when the button was pressed.
    pub location: Point<i32, Physical>,
    /// The state of the buttons and the movement type
    pub state: Mousebind,
}

/// A trait for converting backend types to a MouseEvent type.
#[allow(dead_code)]
pub(crate) trait ButtonEvent {}

/// A ClientMessageEvent sent by the X server.
#[derive(Debug, Clone, Copy)]
pub struct ClientMessageEvent {
    /// The window receiving the message.
    pub window: XWindowID,
    /// Client message data.
    pub data: ClientMessageData,
    /// The type that the data is to be interpreted as.
    pub type_: XAtom,
}

/// The different formats of a Client message's data,
/// as specified by ICCCM.
#[derive(Clone, Copy, Debug)]
pub enum ClientMessageData {
    /// The data should be interpreted as bytes.
    U8([u8; 20]),
    /// The data should be interpreted as words.
    U16([u16; 10]),
    /// The data should be interpreted as doublewords.
    U32([u32; 5]),
}

#[allow(missing_docs)]
impl ClientMessageData {
    #[inline(always)]
    pub fn is_u8(&self) -> bool {
        matches!(self, Self::U8(_))
    }

    #[inline(always)]
    pub fn is_u16(&self) -> bool {
        matches!(self, Self::U16(_))
    }

    pub fn is_u32(&self) -> bool {
        matches!(self, Self::U32(_))
    }
}

use std::convert::TryFrom;

macro_rules! _impl_tryfrom {
    ($t:ty, $count:expr, $variant:expr) => {
        impl TryFrom<&[$t]> for ClientMessageData {
            type Error = std::array::TryFromSliceError;

            fn try_from(data: &[$t]) -> Result<Self, Self::Error> {
                Ok($variant(<[$t; $count]>::try_from(data)?))
            }
        }
    };
}

_impl_tryfrom!(u8, 20, Self::U8);
_impl_tryfrom!(u16, 10, Self::U16);
_impl_tryfrom!(u32, 5, Self::U32);
