use std::collections::HashMap;

use crate::manager::WindowManager;
use crate::x::{
    core::XConn,
    event::KeypressEvent,
};

//* Re-exports
pub mod keysym {
    pub type KeySym = u32;
    pub use x11::keysym::*;
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum ModKey {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum ButtonMask {
    Left,
    Middle,
    Right,
    Button4,
    Button5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonIndex {
    Left,
    Middle,
    Right,
    Button4,
    Button5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseEventKind {
    Motion,
    Press,
    Release,
}

pub type KeyMask = u16;
pub type KeyCode = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keybind {
    pub modmask: KeyMask,
    pub code: KeyCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mousebind {
    pub modmask: KeyMask,
    pub button: ButtonIndex,
    pub kind: MouseEventKind,
}

impl From<KeypressEvent> for Keybind {
    fn from(from: KeypressEvent) -> Keybind {
        Keybind {
            modmask: from.mask, 
            code: from.keycode,
        }
    }
}

pub type Keybinds<X> = HashMap<Keybind, Box<dyn FnMut(&mut WindowManager<X>)>>;

pub fn new_keybinds<X: XConn>() -> Keybinds<X> {
    HashMap::new()
}

pub type Mousebinds<X> = HashMap<Mousebind, Box<dyn FnMut(&mut WindowManager<X>)>>;

pub fn new_mousebinds<X: XConn>() -> Mousebinds<X> {
    HashMap::new()
}

//todo: Add mouse events