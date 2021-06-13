use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::manager::WindowManager;
use crate::x::{
    core::XConn,
    event::KeypressEvent,
};
use crate::{ToaruError, Result};

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

pub const fn kb(modmask: u16, code: u8) -> Keybind {
    Keybind {
        modmask, code
    }
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

/// A type that maps a set of keysyms to a keycode.
/// 
/// Requires `xmodmap` in order to work. Returns SpawnProc error otherwise.
pub struct Keymap {
    map: HashMap<Vec<String>, KeyCode>
}

impl Keymap {
    pub fn new() -> Result<Keymap> {
        let mut map = HashMap::new();
    
        let output = Command::new("xmodmap")
            .arg("-pke")
            .stdout(Stdio::piped())
            .output()?;
    
        let raw_xmod = String::from_utf8_lossy(&output.stdout).into_owned();
    
        for line in raw_xmod.lines() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            assert_eq!(tokens[0], "keycode");
            assert_eq!(tokens[2], "=");
    
            let keycode = tokens[1].parse::<u8>()?;
            let keysyms: Vec<String> = tokens[3..].iter()
                .map(|s| s.to_string())
                .collect();
    
            map.insert(keysyms, keycode);
        }
    
        Ok(Keymap {map})
    }

    /// Parses a string as a keybinding.
    /// 
    /// Follows the format "mod-key"
    /// 
    /// Ctrl = C,
    /// Alt = A,
    /// Meta = M,
    pub fn parse_keybinding(&self, kb: &str) -> Result<Keybind> {
        let mut modifiers: Vec<u16> = Vec::new();
        let mut code = None;
        for token in kb.split("-") {
            match token {
                "C" => {modifiers.push(ModKey::Ctrl.into());}
                "S" => {modifiers.push(ModKey::Shift.into());}
                "A" => {modifiers.push(ModKey::Alt.into());}
                "M" => {modifiers.push(ModKey::Meta.into());}
                n => {code = self.lookup_key(n);}
            }
        }

        let modmask = modifiers.into_iter()
            .fold(0, |acc, n| acc | n);

        if let Some(code) = code {
            Ok(Keybind {modmask, code})
        } else {
            Err(ToaruError::ParseKeybind(kb.into()))
        }
    }

    fn lookup_key(&self, s: &str) -> Option<KeyCode> {
        for (syms, code) in &self.map {
            if syms.contains(&s.to_string()) {
                return Some(*code)
            }
        }
        None
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

#[cfg(test)]
mod tests {
    use super::*;

    const MOD: u16 = xcb::MOD_MASK_4 as u16;
    const SHIFT: u16 = xcb::MOD_MASK_SHIFT as u16;

    #[test]
    fn test_construct_keymap() {
        let map = Keymap::new().unwrap();
    }

    #[test]
    fn test_parse_keybind() {
        let map = Keymap::new().unwrap();

        let modshift_down = map.parse_keybinding("M-S-Down").unwrap();
        let modshift_a = map.parse_keybinding("M-S-a").unwrap();

        assert_eq!(modshift_down, kb(MOD|SHIFT, 116));
        assert_eq!(modshift_a, kb(MOD|SHIFT, 38));
    }
}

//todo: Add mouse events