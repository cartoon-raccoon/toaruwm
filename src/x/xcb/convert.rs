//! Conversions between Toaru and xcb types.

use std::convert::TryFrom;

use strum::*;

use xcb::x;
use xcb::Xid;

use super::id;
use crate::keybinds::{
    ButtonMask,
    ButtonIndex,
    ModKey,
    Mousebind,
    MouseEventKind,
};
use crate::types::{
    Point,
    BorderStyle,
    ClientConfig,
    ClientAttrs,
};
use crate::x::{
    core::{XError, Result},
    event::MouseEvent,
    xcb::XCBConn,
};
use crate::util;

//* conversions for button masks
impl From<ButtonMask> for x::ButtonMask {
    fn from(from: ButtonMask) -> x::ButtonMask {
        use ButtonMask::*;

        match from {
            Left    => x::ButtonMask::N1,
            Middle  => x::ButtonMask::N2,
            Right   => x::ButtonMask::N3,
            Button4 => x::ButtonMask::N4,
            Button5 => x::ButtonMask::N5,
        }
    }
}

impl From<ButtonIndex> for u8 {
    fn from(from: ButtonIndex) -> u8 {
        use ButtonIndex::*;

        match from {
            Left    => x::ButtonIndex::N1 as u8,
            Middle  => x::ButtonIndex::N2 as u8,
            Right   => x::ButtonIndex::N3 as u8,
            Button4 => x::ButtonIndex::N4 as u8,
            Button5 => x::ButtonIndex::N5 as u8,
        }
    }
}

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

//* modifier key conversions
impl From<ModKey> for x::ModMask {
    fn from(from: ModKey) -> x::ModMask {
        use ModKey::*;

        match from {
            Ctrl  => x::ModMask::CONTROL,
            Alt   => x::ModMask::N1,
            Shift => x::ModMask::SHIFT,
            Meta  => x::ModMask::N4,
        }
    }
}

impl From<ModKey> for u16 {
    fn from(from: ModKey) -> u16 {
        x::ModMask::from(from).bits() as u16
    }
}

impl Mousebind {
    /// Express the modifier mask as an xcb-friendly type.
    pub(super) fn modmask(&self) -> x::ModMask {
        self.modmask.iter()
            .map(|u| x::ModMask::from(*u))
            .fold(x::ModMask::empty(), |acc, u| acc | u)
    }
}

impl ModKey {
    /// Tests if a
    pub(super) fn was_held(&self, state: x::KeyButMask) -> bool {
        match *self {
            Self::Ctrl  => state.contains(x::KeyButMask::CONTROL),
            Self::Alt   => state.contains(x::KeyButMask::MOD1),
            Self::Shift => state.contains(x::KeyButMask::SHIFT),
            Self::Meta  => state.contains(x::KeyButMask::MOD4)
        }
    }
}

impl XCBConn {
    /// Converts generic events into mouse events.
    pub(super) fn do_mouse_press(
        &self, ev: x::ButtonPressEvent, rel: bool
    ) -> Result<MouseEvent> {

        let button = ButtonIndex::try_from(ev.detail())?;
        let modmask = ModKey::iter()
            .filter(|m| m.was_held(ev.state()))
            .collect();

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
            }
        })
    }

    pub(super) fn do_mouse_motion(
        &self, ev: x::MotionNotifyEvent
    ) -> Result<MouseEvent> {

        let Some(button) = self.mousemode.get() else {
            //? fixme (account for this instead of returning Err)
            return Err(XError::ConversionError)
        };
        let modmask = ModKey::iter().filter(
            |m| m.was_held(ev.state())
        ).collect();

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
            }
        })
    }
}

impl TryFrom<x::ButtonIndex> for ButtonIndex {
    type Error = XError;

    fn try_from(from: x::ButtonIndex) -> Result<ButtonIndex> {
        match from as u8 {
            1 => Ok(ButtonIndex::Left),
            2 => Ok(ButtonIndex::Middle),
            3 => Ok(ButtonIndex::Right),
            4 => Ok(ButtonIndex::Button4),
            5 => Ok(ButtonIndex::Button5),
            n => Err(XError::OtherError(format!("Unknown mouse button {}", n))),
        }
    }
}

impl From<ButtonIndex> for x::ButtonIndex {
    fn from(from: ButtonIndex) -> x::ButtonIndex {
        match from {
            ButtonIndex::Left    => x::ButtonIndex::N1,
            ButtonIndex::Middle  => x::ButtonIndex::N2,
            ButtonIndex::Right   => x::ButtonIndex::N3,
            ButtonIndex::Button4 => x::ButtonIndex::N4,
            ButtonIndex::Button5 => x::ButtonIndex::N5,
        }
    }
}

// converting ClientConfigs to (u16, u32) slices for xcb
impl From<&ClientConfig> for Vec<x::ConfigWindow> {
    fn from(from: &ClientConfig) -> Vec<x::ConfigWindow> {
        use ClientConfig::*;
        use super::StackMode::*;

        match from {
            BorderWidth(px) => vec![x::ConfigWindow::BorderWidth(*px)],
            Position(geom) => vec![
                    x::ConfigWindow::X(geom.x),
                    x::ConfigWindow::Y(geom.y),
                    x::ConfigWindow::Height(geom.height as u32),
                    x::ConfigWindow::Width(geom.width as u32),
            ],
            Resize {h, w} => vec![
                x::ConfigWindow::Height(*h as u32),
                x::ConfigWindow::Width(*w as u32),
            ],
            Move {x, y} => vec![
                x::ConfigWindow::X(*x),
                x::ConfigWindow::Y(*y),
            ],
            StackingMode(sm) => {
                match sm {
                    Above    => vec![
                        x::ConfigWindow::StackMode(x::StackMode::Above)
                    ],
                    Below    => vec![
                        x::ConfigWindow::StackMode(x::StackMode::Below)
                    ],
                    TopIf    => vec![
                        x::ConfigWindow::StackMode(x::StackMode::TopIf)
                    ],
                    BottomIf => vec![
                        x::ConfigWindow::StackMode(x::StackMode::BottomIf)
                    ],
                    Opposite => vec![
                        x::ConfigWindow::StackMode(x::StackMode::Opposite)
                    ],
                }
            }
        }
    }
}

use x::{Cw, EventMask};

/// Event mask for enabling client events.
pub const ENABLE_CLIENT_EVENTS: EventMask =
    EventMask::ENTER_WINDOW
    .union(EventMask::LEAVE_WINDOW)
    .union(EventMask::PROPERTY_CHANGE)
    .union(EventMask::STRUCTURE_NOTIFY);

/// Event mask for disabling client events.
pub const DISABLE_CLIENT_EVENTS: EventMask =
    EventMask::NO_EVENT;

/// Event mask for selecting events on the root window.
pub const ROOT_EVENT_MASK: EventMask =
    EventMask::PROPERTY_CHANGE
    .union(EventMask::SUBSTRUCTURE_REDIRECT)
    .union(EventMask::SUBSTRUCTURE_NOTIFY)
    .union(EventMask::BUTTON_MOTION);


impl From<&ClientAttrs> for Vec<Cw> {
    fn from(from: &ClientAttrs) -> Vec<Cw> {
        use ClientAttrs::*;
        use BorderStyle::*;

        match from {
            BorderColour(bs) => {
                match bs {
                    Focused   => vec![Cw::BorderPixel(util::FOCUSED_COL)],
                    Unfocused => vec![Cw::BorderPixel(util::UNFOCUSED_COL)],
                    Urgent    => vec![Cw::BorderPixel(util::URGENT_COL)],
                }
            },
            EnableClientEvents => {
                vec![Cw::EventMask(ENABLE_CLIENT_EVENTS)]
            }
            DisableClientEvents => {
                vec![Cw::EventMask(DISABLE_CLIENT_EVENTS)]
            }
            RootEventMask => {
                vec![Cw::EventMask(ROOT_EVENT_MASK)]
            }
        }
    }
}