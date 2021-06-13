use std::convert::TryFrom;

use crate::keybinds::{
    ButtonMask,
    ButtonIndex,
    ModKey,
};
use crate::types::{
    BorderStyle,
    ClientConfig,
    ClientAttrs,
};
use crate::x::core::{XError, Result};
use crate::util;

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
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, geom.height),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, geom.width),
                ]
            }
            Resize {h, w} => vec![
                (xcb::CONFIG_WINDOW_HEIGHT as u16, *h),
                (xcb::CONFIG_WINDOW_WIDTH as u16, *w),
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