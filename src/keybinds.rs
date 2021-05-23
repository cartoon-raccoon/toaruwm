use std::collections::HashMap;

use crate::manager::WindowManager;

//* Re-exports
pub mod keysym {
    pub type KeySym = u32;
    pub use x11::keysym::*;
}

pub type ModMask = u32;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Keybind {
    pub keysym: keysym::KeySym,
    pub modmask: ModMask,
}

pub type Keybinds<X> = HashMap<Keybind, Box<dyn FnMut(&mut WindowManager<X>)>>;