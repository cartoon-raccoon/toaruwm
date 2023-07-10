use std::collections::HashMap;
use std::process::{Command, Stdio};

use strum::*;

use crate::manager::WindowManager;
use crate::types::Point;
use crate::x::{
    core::XConn,
    event::KeypressEvent,
    input::{KeyCode, ModMask},
};
use crate::{Result, ToaruError};

pub use crate::x::input::MouseEventKind;

/// A type representing a modifier key tied to a certain keybind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum ModKey {
    Ctrl = ModMask::CONTROL.bits() as isize,
    Alt = ModMask::MOD1.bits() as isize,
    Shift = ModMask::SHIFT.bits() as isize,
    Meta = ModMask::MOD4.bits() as isize,
}

impl From<Vec<ModKey>> for ModMask {
    fn from(from: Vec<ModKey>) -> ModMask {
        from.into_iter().fold(ModMask::empty(), |acc, n| match n {
            ModKey::Ctrl => acc | ModMask::CONTROL,
            ModKey::Alt => acc | ModMask::MOD1,
            ModKey::Shift => acc | ModMask::SHIFT,
            ModKey::Meta => acc | ModMask::MOD4,
        })
    }
}

/// A type representing a mouse button tied to a certain mousebind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum ButtonIndex {
    Left,
    Middle,
    Right,
    Button4,
    Button5,
}

/// Representation of a Keybind that can be run by ToaruWM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keybind {
    pub(crate) modmask: ModMask,
    pub(crate) code: KeyCode,
}

impl Keybind {
    pub fn new<M: Into<ModMask>>(modifiers: M, code: KeyCode) -> Self {
        Self {
            modmask: modifiers.into(),
            code,
        }
    }
}

/// Representation of a mouse binding that can be run by ToaruWM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mousebind {
    pub(crate) modmask: ModMask,
    pub(crate) button: ButtonIndex,
    pub(crate) kind: MouseEventKind,
}

impl Mousebind {
    pub fn new<M: Into<ModMask>>(modifiers: M, button: ButtonIndex, kind: MouseEventKind) -> Self {
        Self {
            modmask: modifiers.into(),
            button,
            kind,
        }
    }
}

pub fn kb(modmask: Vec<ModKey>, code: u8) -> Keybind {
    Keybind {
        modmask: modmask.into(),
        code,
    }
}

pub fn mb(modmask: Vec<ModKey>, button: ButtonIndex, kind: MouseEventKind) -> Mousebind {
    Mousebind {
        modmask: modmask.into(),
        button,
        kind,
    }
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
    map: HashMap<Vec<String>, KeyCode>,
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
            // format:
            // keycode <byte> = [keysyms]
            let tokens: Vec<&str> = line.split_whitespace().collect();
            assert_eq!(tokens[0], "keycode");
            assert_eq!(tokens[2], "=");

            let keycode = tokens[1].parse::<u8>()?;
            let keysyms: Vec<String> = if tokens.len() > 3 {
                tokens[3..].iter().map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            };

            map.insert(keysyms, keycode);
        }

        Ok(Keymap { map })
    }

    /// Parses a string as a keybinding.
    ///
    /// Follows the format "mod-key"
    ///
    /// Ctrl = C,
    /// Alt = A,
    /// Meta = M,
    pub fn parse_keybinding(&self, kb: &str) -> Result<Keybind> {
        let mut modifiers: Vec<ModKey> = Vec::new();
        let mut code = None;
        for token in kb.split('-') {
            match token {
                "C" => {
                    modifiers.push(ModKey::Ctrl);
                }
                "S" => {
                    modifiers.push(ModKey::Shift);
                }
                "A" => {
                    modifiers.push(ModKey::Alt);
                }
                "M" => {
                    modifiers.push(ModKey::Meta);
                }
                n => {
                    code = self.lookup_key(n);
                }
            }
        }

        if let Some(code) = code {
            Ok(Keybind {
                modmask: modifiers.into(),
                code,
            })
        } else {
            Err(ToaruError::ParseKeybind(kb.into()))
        }
    }

    fn lookup_key(&self, s: &str) -> Option<KeyCode> {
        for (syms, code) in &self.map {
            if syms.contains(&s.to_string()) {
                return Some(*code);
            }
        }
        None
    }
}

/// A set of keybinds that can be run by the the window manager.
///
/// It consists of two components: A keybind, and its associated
/// callback function. It accepts a mutable reference to a
/// WindowManager to run associated methods.
pub type Keybinds<X> = HashMap<Keybind, Box<dyn FnMut(&mut WindowManager<X>)>>;

pub fn new_keybinds<X: XConn>() -> Keybinds<X> {
    HashMap::new()
}

/// A set of mousebinds that can be run by the window manager.
///
/// Like Keybinds, it consists of a mousebind and its associated
/// callback function. It accepts a mutable reference to a WindowManager
/// and a [Point][1], which contains the current coordinates of the pointer.
/// This point is used internally by the WindowManager and should not appear
/// in the user-facing API.
///
/// [1]: crate::core::types::Point
pub type Mousebinds<X> = HashMap<Mousebind, Box<dyn FnMut(&mut WindowManager<X>, Point)>>;

pub fn new_mousebinds<X: XConn>() -> Mousebinds<X> {
    HashMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_keymap() {
        Keymap::new().unwrap();
    }

    #[test]
    fn test_parse_keybind() {
        let map = Keymap::new().unwrap();

        let modshift_down = map.parse_keybinding("M-S-Down").unwrap();
        let modshift_a = map.parse_keybinding("M-S-a").unwrap();

        let mod4 = ModKey::Meta;
        let shift = ModKey::Shift;

        assert_eq!(modshift_down, kb(vec![mod4, shift], 116));
        assert_eq!(modshift_a, kb(vec![mod4, shift], 38));
    }
}
