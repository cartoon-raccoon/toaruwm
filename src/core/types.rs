pub mod keysym {
    pub use x11::keysym::*;
}

pub use xcb::ModMask as ModMask;

pub use crate::core::{Ring, Selector};

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