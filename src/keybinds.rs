use std::collections::HashMap;

use crate::manager::WindowManager;
use crate::x::event::KeypressEvent;

//* Re-exports
pub mod keysym {
    pub type KeySym = u32;
    pub use x11::keysym::*;
}

pub type ModMask = u32;
pub type ButtonMask = u32;
pub type ButtonIndex = u32;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Keybind {
    pub modmask: ModMask,
    pub keysym: keysym::KeySym,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Mousebind {
    pub button: ButtonIndex,
    pub mask: ButtonMask,
    pub modmask: ModMask,
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