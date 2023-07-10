//! Conversions between Toaru and xcb types.

use std::convert::TryFrom;

use strum::*;

use xcb::x;
use xcb::Xid;

use super::{id, XCBConn};
use crate::keybinds::{ButtonIndex, ModKey, Mousebind};
use crate::types::{BorderStyle, ClientAttrs, ClientConfig, Point};
use crate::util;
use crate::x::{
    core::{BitMask, Result, XError},
    event::MouseEvent,
    input::{ButtonMask, KeyButMask, ModMask, MouseEventKind},
};

impl BitMask for x::ModMask {}
impl BitMask for x::ButtonMask {}
impl BitMask for x::KeyButMask {}

//* mouse button and button index conversions
impl From<ButtonMask> for x::ButtonMask {
    fn from(from: ButtonMask) -> x::ButtonMask {
        x::ButtonMask::from_bits_truncate(from.bits() as u32)
    }
}

impl From<ButtonIndex> for x::ButtonIndex {
    fn from(from: ButtonIndex) -> x::ButtonIndex {
        use ButtonIndex::*;

        match from {
            Left => x::ButtonIndex::N1,
            Middle => x::ButtonIndex::N2,
            Right => x::ButtonIndex::N3,
            Button4 => x::ButtonIndex::N4,
            Button5 => x::ButtonIndex::N5,
        }
    }
}

//* modifier key conversions
impl From<ModKey> for x::ModMask {
    fn from(from: ModKey) -> x::ModMask {
        use ModKey::*;

        match from {
            Ctrl => x::ModMask::CONTROL,
            Alt => x::ModMask::N1,
            Shift => x::ModMask::SHIFT,
            Meta => x::ModMask::N4,
        }
    }
}

impl From<ModMask> for x::ModMask {
    fn from(from: ModMask) -> x::ModMask {
        x::ModMask::from_bits_truncate(from.bits() as u32)
    }
}

impl From<x::ModMask> for ModMask {
    fn from(from: x::ModMask) -> ModMask {
        ModMask::from_bits_truncate(from.bits() as u16)
    }
}

impl From<KeyButMask> for x::KeyButMask {
    fn from(from: KeyButMask) -> x::KeyButMask {
        x::KeyButMask::from_bits_truncate(from.bits() as u32)
    }
}

impl From<x::KeyButMask> for KeyButMask {
    fn from(from: x::KeyButMask) -> KeyButMask {
        KeyButMask::from_bits_truncate(from.bits() as u16)
    }
}

impl From<x::KeyButMask> for ModMask {
    fn from(from: x::KeyButMask) -> ModMask {
        KeyButMask::from(from).modmask()
    }
}

impl XCBConn {
    /// Converts generic events into mouse events.
    pub(super) fn do_mouse_press(&self, ev: x::ButtonPressEvent, rel: bool) -> Result<MouseEvent> {
        let button = ButtonIndex::try_from(ev.detail())?;
        let modmask = ModKey::iter()
            .filter(|m| m.was_held(ev.state()))
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
            id: id!(ev.child()),
            location: Point {
                x: ev.root_x() as i32,
                y: ev.root_y() as i32,
            },
            state: Mousebind {
                button,
                modmask,
                kind,
            },
        })
    }

    pub(super) fn do_mouse_motion(&self, ev: x::MotionNotifyEvent) -> Result<MouseEvent> {
        let Some(button) = self.mousemode.get() else {
            //? fixme (account for this instead of returning Err)
            return Err(XError::ConversionError)
        };
        let modmask = ModKey::iter()
            .filter(|m| m.was_held(ev.state()))
            .fold(ModMask::empty(), |acc, n| {
                acc | <ModKey as Into<ModMask>>::into(n)
            });

        Ok(MouseEvent {
            id: id!(ev.child()),
            location: Point {
                x: ev.root_x() as i32,
                y: ev.root_y() as i32,
            },
            state: Mousebind {
                button,
                modmask,
                kind: MouseEventKind::Motion,
            },
        })
    }
}

// converting ClientConfigs to (u16, u32) slices for xcb
impl From<&ClientConfig> for Vec<x::ConfigWindow> {
    fn from(from: &ClientConfig) -> Vec<x::ConfigWindow> {
        use super::StackMode::*;
        use ClientConfig::*;

        match from {
            BorderWidth(px) => vec![x::ConfigWindow::BorderWidth(*px)],
            Position(geom) => vec![
                x::ConfigWindow::X(geom.x),
                x::ConfigWindow::Y(geom.y),
                x::ConfigWindow::Height(geom.height as u32),
                x::ConfigWindow::Width(geom.width as u32),
            ],
            Resize { h, w } => vec![
                x::ConfigWindow::Height(*h as u32),
                x::ConfigWindow::Width(*w as u32),
            ],
            Move { x, y } => vec![x::ConfigWindow::X(*x), x::ConfigWindow::Y(*y)],
            StackingMode(sm) => match sm {
                Above => vec![x::ConfigWindow::StackMode(x::StackMode::Above)],
                Below => vec![x::ConfigWindow::StackMode(x::StackMode::Below)],
                TopIf => vec![x::ConfigWindow::StackMode(x::StackMode::TopIf)],
                BottomIf => vec![x::ConfigWindow::StackMode(x::StackMode::BottomIf)],
                Opposite => vec![x::ConfigWindow::StackMode(x::StackMode::Opposite)],
            },
        }
    }
}

use x::{Cw, EventMask};

/// Event mask for enabling client events.
pub const ENABLE_CLIENT_EVENTS: EventMask = EventMask::ENTER_WINDOW
    .union(EventMask::LEAVE_WINDOW)
    .union(EventMask::PROPERTY_CHANGE)
    .union(EventMask::STRUCTURE_NOTIFY);

/// Event mask for disabling client events.
pub const DISABLE_CLIENT_EVENTS: EventMask = EventMask::NO_EVENT;

/// Event mask for selecting events on the root window.
pub const ROOT_EVENT_MASK: EventMask = EventMask::PROPERTY_CHANGE
    .union(EventMask::SUBSTRUCTURE_REDIRECT)
    .union(EventMask::SUBSTRUCTURE_NOTIFY)
    .union(EventMask::BUTTON_MOTION);

impl From<&ClientAttrs> for Cw {
    fn from(from: &ClientAttrs) -> Cw {
        use BorderStyle::*;
        use ClientAttrs::*;

        match from {
            BorderColour(bs) => match bs {
                Focused => Cw::BorderPixel(util::FOCUSED_COL),
                Unfocused => Cw::BorderPixel(util::UNFOCUSED_COL),
                Urgent => Cw::BorderPixel(util::URGENT_COL),
            },
            EnableClientEvents => Cw::EventMask(ENABLE_CLIENT_EVENTS),
            DisableClientEvents => Cw::EventMask(DISABLE_CLIENT_EVENTS),
            RootEventMask => Cw::EventMask(ROOT_EVENT_MASK),
        }
    }
}
