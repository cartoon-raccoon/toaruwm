use std::collections::HashMap;

use crate::manager::WindowManager;
use crate::x::event::KeypressEvent;

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

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum ButtonIndex {
    Left,
    Middle,
    Right,
    Button4,
    Button5,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Keybind {
    pub modmask: ModKey,
    pub keysym: keysym::KeySym,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Mousebind {
    pub button: ButtonIndex,
    pub mask: ButtonMask,
    pub modmask: ModKey,
}

impl From<KeypressEvent> for Keybind {
    fn from(from: KeypressEvent) -> Keybind {
        Keybind {
            modmask: from.mask, 
            keysym: from.keysym,
        }
    }
}

pub type Keybinds<X> = HashMap<Keybind, Box<dyn FnMut(&mut WindowManager<X>)>>;

//todo: Add mouse events