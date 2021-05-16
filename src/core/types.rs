use std::ops::Deref;

use thiserror::Error;

use crate::layouts::LayoutType;

pub mod keysym {
    pub use x11::keysym::*;
}

pub use xcb::ModMask as ModMask;

pub use crate::core::{Ring, Selector};

pub type Atom = u32;

pub type Result<T> = ::core::result::Result<T, WMError>;

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
            width: 160,
            height: 100,
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SizeHints {
    pub position: Option<(i32, i32)>,
    pub size: Option<(i32, i32)>,
    pub min_size: Option<(i32, i32)>,
    pub max_size: Option<(i32, i32)>,
    pub resize: Option<(i32, i32)>,
    pub min_aspect: Option<(i32, i32)>,
    pub max_aspect: Option<(i32, i32)>,
    pub base: Option<(i32, i32)>,
    pub gravity: Option<u32>
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

/// Convenience wrapper around a Vec of NetWindowStates.
#[derive(Debug, Clone)]
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

/// ICCCM-defined window hints.
#[derive(Debug, Clone, Copy)]
pub struct WmHints {
    pub state: WindowState,
    pub urgent: bool,
    //todo: add pixmaps
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