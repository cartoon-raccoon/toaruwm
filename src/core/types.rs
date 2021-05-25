use std::ops::Deref;

use thiserror::Error;

use crate::layouts::LayoutType;
//use crate::manager::WindowManager;

pub use crate::core::{Ring, Selector};
pub use crate::x::core::{WmHints, SizeHints};

pub type ModMask = u32;
pub type Atom = u32;

pub use super::window::{Client, ClientRing};

pub type Result<T> = ::core::result::Result<T, WMError>;

// todo: deprecate this and put inside configuration
pub const BORDER_WIDTH: u32 = 2;

#[derive(Debug, Error, Clone, Copy)]
pub enum WMError {
    
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub height: u32,
    pub width: u32,
}

// impl From<(i32, i32, i32, i32)> for Geometry {
//     fn from(from: (i32, i32, i32, i32)) -> Self {
//         Self {
//             x: from.0,
//             y: from.1,
//             width: from.2,
//             height: from.3,
//         }
//     }
// }

impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            x: 0,
            y: 0,
            height: 100,
            width: 160,
        }
    }
}

impl Geometry {
    pub fn zeroed() -> Self {
        Geometry {
            x: 0,
            y: 0,
            height: 0,
            width: 0,
        }
    }
}

// Whether the mouse button is pressed.
#[derive(Debug, Clone, Copy)]
pub enum MouseMode {
    None,
    Move,
    Resize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum WinLayoutState {
    Tiled,
    Floating,
}

// The ICCCM-defined window states.
#[derive(Clone, Copy, Debug)]
pub enum WindowState {
    Normal,
    Iconic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderStyle {
    Focused,
    Unfocused,
    Urgent,
}

/// Convenience wrapper around a Vec of NetWindowStates.
#[derive(Debug, Clone, Default)]
pub struct NetWindowStates {
    states: Vec<Atom>,
}

impl NetWindowStates {
    pub fn new() -> Self {
        Self {
            states: Vec::new()
        }
    }

    pub fn contains(&self, prop: Atom) -> bool {
        self.states.contains(&prop)
    }

    pub fn add(&mut self, prop: Atom) {
        self.states.push(prop)
    }

    pub fn remove(&mut self, prop: Atom) -> Atom {
        for (idx, atom) in self.states.iter().enumerate() {
            if *atom == prop {
                return self.states.remove(idx)
            }
        }
        //error!("Tried to remove atom not in states");
        0
    }
}

impl From<Vec<Atom>> for NetWindowStates {
    fn from(from: Vec<Atom>) -> Self {
        Self {
            states: from
        }
    }
}

impl Deref for NetWindowStates {
    type Target = [Atom];

    fn deref(&self) -> &Self::Target {
        self.states.as_slice()
    }
}

impl From<LayoutType> for WinLayoutState {

    #[inline]
    fn from(from: LayoutType) -> WinLayoutState {
        if let LayoutType::Floating = from {
            return Self::Floating
        }

        Self::Tiled
    }
}

/// ICCCM-defined window properties.
//todo: make all fields private, accessible with methods.
pub struct XWinProperties {
    pub(crate) wm_name: String,
    pub(crate) wm_icon_name: String,
    pub(crate) wm_size_hints: Option<SizeHints>,
    pub(crate) wm_hints: Option<WmHints>,
    pub(crate) wm_class: (String, String), //Instance, Class
    pub(crate) wm_protocols: Option<Vec<Atom>>,
    pub(crate) wm_state: WindowState,
}

impl XWinProperties {
    pub fn wm_name(&self) -> &str {
        &self.wm_name
    }

    pub fn wm_icon_name(&self) -> &str {
        &self.wm_icon_name
    }

    #[inline]
    pub fn wm_size_hints(&self) -> Option<&SizeHints> {
        self.wm_size_hints.as_ref()
    }

    pub fn wm_hints(&self) -> Option<&WmHints> {
        self.wm_hints.as_ref()
    }

    pub fn wm_class(&self) -> (&str, &str) {
        (&self.wm_class.0, &self.wm_class.1)
    }

    pub fn window_type(&self) -> Option<&[Atom]> {
        self.wm_protocols.as_deref()
    }

    pub fn wm_state(&self) -> WindowState {
        self.wm_state
    }
}