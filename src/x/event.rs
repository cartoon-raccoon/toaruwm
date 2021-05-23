use super::{XWindowID};
use crate::types::{
    Geometry, Point, Atom,
};
use crate::keybinds::{
    keysym::KeySym,
    ModMask,
};

/// Low-level wrapper around actual X server events.
/// 
/// Translated to EventActions by `WindowManager`.
#[derive(Debug, Clone, Copy)]
pub enum XEvent {
    /// Notification that a client has changed its configuration.
    ConfigureNotify(ConfigureEvent),
    /// Request for configuration from a client.
    ConfigureRequest(ConfigureEvent),
    /// A Client is requesting to be mapped.
    MapRequest(XWindowID, bool), // bool: override_redirect
    /// A Client has mapped a window.
    MapNotify(XWindowID),
    /// A Client has unmapped a window.
    UnmapNotify(XWindowID),
    /// A Client has destroyed a window.
    DestroyNotify(XWindowID),
    /// The pointer has entered a window.
    EnterNotify(XWindowID),
    /// The pointer has left a window.
    LeaveNotify(XWindowID),
    /// The mouse has moved.
    MotionNotify(Point),
    /// A window was reparented.
    ReparentNotify(XWindowID),
    /// A window property was changed.
    PropertyNotify(PropertyEvent),
    /// A key combination was pressed.
    KeyPress(KeypressEvent),
    /// A key combination was released.
    KeyRelease,
    /// A mouse button was pressed.
    ButtonPress(XWindowID, Point),
    /// A mouse button was released.
    ButtonRelease,
    /// A client message was received.
    ClientMessage(XWindowID, ClientMessageData),
    /// Unknown event type, used as a catchall for events not tracked by toaruwm.
    Unknown(u8),
}

/// Data associated with a configure event.
#[derive(Debug, Clone, Copy)]
pub struct ConfigureEvent {
    /// The window associated with the event.
    pub id: XWindowID,
    /// The new geometry requested by the window.
    pub geom: Geometry,
    /// Is the window the root window
    pub is_root: bool,
}

/// Data associated with a property change event.
#[derive(Debug, Clone, Copy)]
pub struct PropertyEvent {
    /// The window associated with the event.
    pub id: XWindowID,
    /// The atom representing the change.
    pub atom: Atom,
    /// The time of event.
    pub time: u32,
}

/// Data associated with a key press event.
#[derive(Debug, Clone, Copy)]
pub struct KeypressEvent {
    /// What modmask was active at the time.
    pub mask: ModMask,
    /// The key pressed.
    pub keysym: KeySym,
}

/// The different formats of a Client message's data,
/// as specified by ICCCM.
#[derive(Clone, Copy, Debug)]
pub enum ClientMessageData {
    U8([u8; 20]),
    U16([u16; 10]),
    U32([u32; 5]),
}

impl ClientMessageData {
    //todo: move this to the xcb conn impl
    // pub fn from_event(event: &xproto::ClientMessageEvent) -> Self {
    //     let data = event.data();
    //     match event.format() {
    //         8 => {
    //             Self::U8(data.data8()[0..20]
    //             .try_into().expect("Byte: Incorrect conversion"))
    //         }
    //         16 => {
    //             Self::U16(data.data16()[0..10]
    //             .try_into().expect("Word: Incorrect conversion"))
    //         }
    //         32 => {
    //             Self::U32(data.data32()[0..5]
    //             .try_into().expect("DWord: Incorrect conversion"))
    //         }
    //         _ => {unreachable!()}
    //     }
    // }

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