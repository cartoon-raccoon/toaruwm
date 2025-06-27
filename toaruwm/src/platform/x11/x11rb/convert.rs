//! Conversions between Toaru and xcb types.

use std::convert::TryFrom;

use x11rb::protocol::xproto;

use strum::*;

use super::Initialized;
use crate::platform::x11::types::{ClientAttrs, ClientConfig};
use crate::bindings::{ButtonIndex, ModKey, Mousebind};
use crate::types::{Point};
use crate::platform::x11::{
    core::{Result, XError, Xid},
    event::MouseEvent,
    input::{ButtonMask, KeyButMask, ModMask, MouseEventKind},
    x11rb::X11RBConn,
};

//* button mask and index conversions
#[doc(hidden)]
impl From<ButtonMask> for xproto::ButtonMask {
    fn from(from: ButtonMask) -> xproto::ButtonMask {
        xproto::ButtonMask::from(from.bits())
    }
}

#[doc(hidden)]
impl From<xproto::ButtonMask> for ButtonMask {
    fn from(from: xproto::ButtonMask) -> ButtonMask {
        ButtonMask::from_bits_truncate(from.bits())
    }
}

#[doc(hidden)]
impl From<ButtonIndex> for xproto::ButtonIndex {
    fn from(from: ButtonIndex) -> xproto::ButtonIndex {
        use ButtonIndex::*;

        match from {
            Left => xproto::ButtonIndex::M1,
            Middle => xproto::ButtonIndex::M2,
            Right => xproto::ButtonIndex::M3,
            Button4 => xproto::ButtonIndex::M4,
            Button5 => xproto::ButtonIndex::M5,
        }
    }
}

//* modifier key conversions
#[doc(hidden)]
impl From<ModKey> for xproto::ModMask {
    fn from(from: ModKey) -> xproto::ModMask {
        use ModKey::*;

        match from {
            Ctrl => xproto::ModMask::CONTROL,
            Alt => xproto::ModMask::M1,
            Shift => xproto::ModMask::SHIFT,
            Meta => xproto::ModMask::M4,
        }
    }
}

#[doc(hidden)]
impl From<ModMask> for xproto::ModMask {
    fn from(from: ModMask) -> xproto::ModMask {
        xproto::ModMask::from(from.bits())
    }
}

#[doc(hidden)]
impl From<xproto::ModMask> for ModMask {
    fn from(from: xproto::ModMask) -> ModMask {
        ModMask::from_bits_truncate(from.bits())
    }
}

//* conversions for keybutmask
#[doc(hidden)]
impl From<KeyButMask> for xproto::KeyButMask {
    fn from(from: KeyButMask) -> xproto::KeyButMask {
        xproto::KeyButMask::from(from.bits())
    }
}

#[doc(hidden)]
impl From<xproto::KeyButMask> for KeyButMask {
    fn from(from: xproto::KeyButMask) -> KeyButMask {
        KeyButMask::from_bits_truncate(from.bits())
    }
}

impl X11RBConn<Initialized> {
    /// Converts generic events into mouse events.
    pub(in crate::platform::x11::x11rb) fn do_mouse_press(
        &self,
        ev: xproto::ButtonPressEvent,
        rel: bool,
    ) -> Result<MouseEvent> {
        let button = ButtonIndex::try_from(ev.detail)?;
        let modmask = ModKey::iter()
            .filter(|m| m.was_held(KeyButMask::from(ev.state).modmask()))
            .fold(ModMask::empty(), |acc, n| {
                acc | <ModKey as Into<ModMask>>::into(n)
            });

        let kind = if !rel {
            self.mousemode.set(Some(button));
            MouseEventKind::Press
        } else {
            self.mousemode.set(None);
            MouseEventKind::Release
        };

        Ok(MouseEvent {
            id: Xid(ev.child),
            location: Point::new(ev.root_x as i32, ev.root_y as i32),
            state: Mousebind {
                button,
                modmask,
                kind,
            },
        })
    }

    pub(in crate::platform::x11::x11rb) fn do_mouse_motion(
        &self,
        ev: xproto::MotionNotifyEvent,
    ) -> Result<MouseEvent> {
        let Some(button) = self.mousemode.get() else {
            //? fixme (account for this instead of returning Err)
            return Err(XError::ConversionError)
        };
        let modmask = ModKey::iter()
            .filter(|m| m.was_held(KeyButMask::from(ev.state).modmask()))
            .fold(ModMask::empty(), |acc, n| {
                acc | <ModKey as Into<ModMask>>::into(n)
            });

        Ok(MouseEvent {
            id: Xid(ev.child),
            location: Point::new(ev.root_x as i32, ev.root_y as i32),
            state: Mousebind {
                button,
                modmask,
                kind: MouseEventKind::Motion,
            },
        })
    }
}

use xproto::{ConfigureWindowAux, StackMode};
// converting ClientConfigs to (u16, u32) slices for xcb
impl From<&ClientConfig> for ConfigureWindowAux {
    fn from(from: &ClientConfig) -> ConfigureWindowAux {
        use super::StackMode::*;
        use ClientConfig::*;

        match from {
            BorderWidth(px) => ConfigureWindowAux::new().border_width(*px),
            Position(geom) => ConfigureWindowAux::new()
                .x(geom.point.x)
                .y(geom.point.y)
                .width(geom.size.width as u32)
                .height(geom.size.height as u32),
            Resize { h, w } => ConfigureWindowAux::new().height(*h as u32).width(*w as u32),
            Move { x, y } => ConfigureWindowAux::new().x(*x).y(*y),
            StackingMode(sm) => {
                let new = ConfigureWindowAux::new();
                match sm {
                    Above(sib) => new
                        .stack_mode(StackMode::ABOVE)
                        .sibling(sib.map(|s| s.val())),
                    Below(sib) => new
                        .stack_mode(StackMode::BELOW)
                        .sibling(sib.map(|s| s.val())),
                    TopIf(sib) => new
                        .stack_mode(StackMode::TOP_IF)
                        .sibling(sib.map(|s| s.val())),
                    BottomIf(sib) => new
                        .stack_mode(StackMode::BOTTOM_IF)
                        .sibling(sib.map(|s| s.val())),
                    Opposite(sib) => new
                        .stack_mode(StackMode::OPPOSITE)
                        .sibling(sib.map(|s| s.val())),
                }
            }
        }
    }
}

use x11rb::protocol::xproto::{ChangeWindowAttributesAux, EventMask};

macro_rules! enable_client_events {
    () => {
        EventMask::ENTER_WINDOW
            | EventMask::LEAVE_WINDOW
            | EventMask::PROPERTY_CHANGE
            | EventMask::STRUCTURE_NOTIFY
    };
}

macro_rules! disable_client_events {
    () => {
        EventMask::NO_EVENT
    };
}

macro_rules! root_event_mask {
    () => {
        EventMask::PROPERTY_CHANGE
            | EventMask::SUBSTRUCTURE_REDIRECT
            | EventMask::SUBSTRUCTURE_NOTIFY
            | EventMask::BUTTON_MOTION
    };
}

impl From<&ClientAttrs> for ChangeWindowAttributesAux {
    fn from(from: &ClientAttrs) -> ChangeWindowAttributesAux {
        use ClientAttrs::*;

        let new = ChangeWindowAttributesAux::new();
        match from {
            BorderColour(c) => new.border_pixel(c.as_u32()),
            EnableClientEvents => new.event_mask(enable_client_events!()),
            DisableClientEvents => new.event_mask(disable_client_events!()),
            RootEventMask => new.event_mask(root_event_mask!()),
        }
    }
}

pub(super) fn convert_cws(attrs: &[ClientAttrs]) -> ChangeWindowAttributesAux {
    use ClientAttrs::*;

    let new = ChangeWindowAttributesAux::new();
    attrs.iter().fold(new, |cw, attr| match *attr {
        BorderColour(c) => cw.border_pixel(c.as_u32()),
        EnableClientEvents => cw.event_mask(enable_client_events!()),
        DisableClientEvents => cw.event_mask(disable_client_events!()),
        RootEventMask => cw.event_mask(root_event_mask!()),
    })
}
