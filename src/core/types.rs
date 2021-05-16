use crate::layouts::LayoutType;

pub mod keysym {
    pub use x11::keysym::*;
}

pub use xcb::ModMask as ModMask;

pub use crate::core::{Ring, Selector};

pub type Atom = u32;

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