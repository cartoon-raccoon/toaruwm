//! Conversions between Toaru and xcb types.

use std::convert::{TryFrom, TryInto};

use strum::*;

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
    xcb::{X_EVENT_MASK, XCBConn},
};
use crate::util;

//* conversions for button masks
impl From<ButtonMask> for xcb::ButtonMask {
    fn from(from: ButtonMask) -> xcb::ButtonMask {
        use ButtonMask::*;

        match from {
            Left => xcb::BUTTON_MASK_1,
            Middle => xcb::BUTTON_MASK_2,
            Right => xcb::BUTTON_MASK_3,
            Button4 => xcb::BUTTON_MASK_4,
            Button5 => xcb::BUTTON_MASK_5,
        }
    }
}

impl From<ButtonIndex> for u8 {
    fn from(from: ButtonIndex) -> u8 {
        use ButtonIndex::*;

        match from {
            Left => xcb::BUTTON_INDEX_1 as u8,
            Middle => xcb::BUTTON_INDEX_2 as u8,
            Right => xcb::BUTTON_INDEX_3 as u8,
            Button4 => xcb::BUTTON_INDEX_4 as u8,
            Button5 => xcb::BUTTON_INDEX_5 as u8,
        }
    }
}

//* modifier key conversions
impl From<ModKey> for u16 {
    fn from(from: ModKey) -> u16 {
        use ModKey::*;

        match from {
            Ctrl => xcb::MOD_MASK_CONTROL as u16,
            Alt => xcb::MOD_MASK_1 as u16,
            Shift => xcb::MOD_MASK_SHIFT as u16,
            Meta => xcb::MOD_MASK_4 as u16,
        }
    }
}

impl Mousebind {
    /// Express the modifier mask as an xcb-friendly type.
    pub(super) fn modmask(&self) -> u16 {
        self.modmask.iter()
            .map(|u| u16::from(*u))
            .fold(0, |acc, u| acc | u)
    }
}

impl ModKey {
    fn was_held(&self, state: u16) -> bool {
        state & u16::from(*self) > 0
    }
}

impl XCBConn {
    /// Converts generic events into mouse events.
    pub(super) fn mouse_event_from_generic(&self, ev: &xcb::GenericEvent) -> Result<MouseEvent> {
        match ev.response_type() & X_EVENT_MASK {
            xcb::BUTTON_PRESS => {
                let ev = unsafe {xcb::cast_event::<xcb::ButtonPressEvent>(ev)};

                let button: ButtonIndex = (ev.detail() as u32).try_into()?;
                self.mousemode.set(Some(button));

                let modmask = ModKey::iter().filter(|m| m.was_held(ev.state() as u16)).collect();

                Ok(MouseEvent {
                    id: ev.child(),
                    location: Point {
                        x: ev.root_x() as i32,
                        y: ev.root_y() as i32,
                    },
                    state: Mousebind {
                        button,
                        modmask,
                        kind: MouseEventKind::Press,
                    }
                })
            }
            xcb::BUTTON_RELEASE => {
                let ev = unsafe {xcb::cast_event::<xcb::ButtonReleaseEvent>(ev)};

                self.mousemode.set(None);

                let modmask = ModKey::iter().filter(|m| m.was_held(ev.state() as u16)).collect();

                Ok(MouseEvent {
                    id: ev.child(),
                    location: Point {
                        x: ev.root_x() as i32,
                        y: ev.root_y() as i32,
                    },
                    state: Mousebind {
                        button: (ev.detail() as u32).try_into()?,
                        modmask,
                        kind: MouseEventKind::Release,
                    }
                })
            }
            xcb::MOTION_NOTIFY => {
                let ev = unsafe {xcb::cast_event::<xcb::MotionNotifyEvent>(ev)};

                //* should be safe to unwrap here since
                //* we only get motion events if a button is pressed
                //? fixme?
                let button = self.mousemode.get().unwrap();
                let modmask = ModKey::iter().filter(|m| m.was_held(ev.state() as u16)).collect();

                Ok(MouseEvent {
                    id: ev.child(),
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
            _ => {
                Err(XError::ConversionError)
            }
        }
    }
}

impl TryFrom<u32> for ButtonIndex {
    type Error = XError;

    fn try_from(from: u32) -> Result<ButtonIndex> {
        match from {
            xcb::BUTTON_INDEX_1 => Ok(ButtonIndex::Left),
            xcb::BUTTON_INDEX_2 => Ok(ButtonIndex::Middle),
            xcb::BUTTON_INDEX_3 => Ok(ButtonIndex::Right),
            xcb::BUTTON_INDEX_4 => Ok(ButtonIndex::Button4),
            xcb::BUTTON_INDEX_5 => Ok(ButtonIndex::Button5),
            n => Err(XError::OtherError(format!("Unknown mouse button {}", n))),
        }
    }
}

// converting ClientConfigs to (u16, u32) slices for xcb
impl From<&ClientConfig> for Vec<(u16, u32)> {
    fn from(from: &ClientConfig) -> Vec<(u16, u32)> {
        use ClientConfig::*;
        use super::StackMode::*;

        match from {
            BorderWidth(px) => vec![(xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, *px)],
            Position(geom) => {
                vec![
                    (xcb::CONFIG_WINDOW_X as u16, geom.x as u32),
                    (xcb::CONFIG_WINDOW_Y as u16, geom.y as u32),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, geom.height as u32),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, geom.width as u32),
                ]
            }
            Resize {h, w} => vec![
                (xcb::CONFIG_WINDOW_HEIGHT as u16, *h as u32),
                (xcb::CONFIG_WINDOW_WIDTH as u16, *w as u32),
            ],
            Move {x, y} => vec![
                (xcb::CONFIG_WINDOW_X as u16, *x as u32),
                (xcb::CONFIG_WINDOW_Y as u16, *y as u32),
            ],
            StackingMode(sm) => {
                let stackmode = xcb::CONFIG_WINDOW_STACK_MODE as u16;
                match sm {
                    Above => vec![(stackmode, xcb::STACK_MODE_ABOVE)],
                    Below => vec![(stackmode, xcb::STACK_MODE_BELOW)],
                    TopIf => vec![(stackmode, xcb::STACK_MODE_TOP_IF)],
                    BottomIf => vec![(stackmode, xcb::STACK_MODE_BOTTOM_IF)],
                    Opposite => vec![(stackmode, xcb::STACK_MODE_OPPOSITE)],
                }
            }
        }
    }
}

impl From<&ClientAttrs> for Vec<(u32, u32)> {
    fn from(from: &ClientAttrs) -> Vec<(u32, u32)> {
        use ClientAttrs::*;
        use BorderStyle::*;

        match from {
            BorderColour(bs) => {
                let bcolour = xcb::CW_BORDER_PIXEL;
                match bs {
                    Focused => vec![(bcolour, util::FOCUSED_COL)],
                    Unfocused => vec![(bcolour, util::UNFOCUSED_COL)],
                    Urgent => vec![(bcolour, util::URGENT_COL)],
                }
            },
            EnableClientEvents => {
                let clienteventmask = xcb::EVENT_MASK_ENTER_WINDOW
                    | xcb::EVENT_MASK_LEAVE_WINDOW
                    | xcb::EVENT_MASK_PROPERTY_CHANGE
                    | xcb::EVENT_MASK_STRUCTURE_NOTIFY;
                
                vec![(xcb::CW_EVENT_MASK, clienteventmask)]
            }
            DisableClientEvents => {
                let clienteventmask = xcb::EVENT_MASK_NO_EVENT;

                vec![(xcb::CW_EVENT_MASK, clienteventmask)]
            }
            RootEventMask => {
                let rooteventmask = xcb::EVENT_MASK_PROPERTY_CHANGE
                    | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT
                    | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY
                    | xcb::EVENT_MASK_BUTTON_MOTION;
                
                vec![(xcb::CW_EVENT_MASK, rooteventmask)]
            }
        }
    }
}