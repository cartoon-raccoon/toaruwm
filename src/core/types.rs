pub mod keysym {
    pub use x11::keysym::*;
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy. PartialEq)]
pub struct Geom {
    pub x: u32,
    pub y: u32,
    pub height: u32,
    pub width: u32,
}