//! Types for parsing and creating key and mouse bindings.

use std::collections::HashMap;
use std::process::{Command, Stdio};

use strum::*;

use custom_debug_derive::Debug;

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
    /// The Ctrl key.
    Ctrl = ModMask::CONTROL.bits() as isize,
    /// The Alt key.
    Alt = ModMask::MOD1.bits() as isize,
    /// The Shift key.
    Shift = ModMask::SHIFT.bits() as isize,
    /// The Super/Meta key.
    Meta = ModMask::MOD4.bits() as isize,
}

// impl<I: IntoIterator<Item = ModKey>> From<I> for ModMask {
//     fn from(from: I) -> ModMask {
//         from.into_iter().fold(ModMask::empty(), |acc, n| match n {
//             ModKey::Ctrl => acc | ModMask::CONTROL,
//             ModKey::Alt => acc | ModMask::MOD1,
//             ModKey::Shift => acc | ModMask::SHIFT,
//             ModKey::Meta => acc | ModMask::MOD4,
//         })
//     }
// }
#[doc(hidden)]
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
    /// The left mouse button.
    Left,
    /// The middle mouse button (clicking the scroll wheel).
    Middle,
    /// The right mouse button.
    Right,
    /// The scroll wheel (direction TODO).
    Button4,
    /// The scroll wheel (direction TODO).
    Button5,
}

/// Representation of a Keybind that can be run by ToaruWM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keybind {
    pub(crate) modmask: ModMask,
    pub(crate) code: KeyCode,
}

impl Keybind {
    /// Creates new `Keybind`.
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
    /// Creates a new Mousebind.
    pub fn new<M: Into<ModMask>>(modifiers: M, button: ButtonIndex, kind: MouseEventKind) -> Self {
        Self {
            modmask: modifiers.into(),
            button,
            kind,
        }
    }
}

/// Convenience function for constructing a keybind.
pub fn kb(modmask: Vec<ModKey>, code: u8) -> Keybind {
    Keybind {
        modmask: modmask.into(),
        code,
    }
}

/// Convenience function for constructing a mousebind.
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Keymap {
    map: HashMap<Vec<String>, KeyCode>,
}

impl Keymap {
    /// Creates a new keymap.
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

    // todo: create a keymap that doesn't rely on xmodmap

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

// macro_rules! _impl_bindings {
//     ($inner:expr, $bind:ty) => {

//     };
// }

//todo
/// An ergonomic wrapper for creating a [`Keybind`].
#[macro_export]
macro_rules! keybind {
    () => {};
}

//todo
/// An ergonomic wrapper for creating a [`Mousebind`].
#[macro_export]
macro_rules! mousebind {
    () => {};
}

//todo: create a macro for tying everything together.

/// A function is run when a keybind is invoked.
pub type KeyCallback<X> = Box<dyn FnMut(&mut WindowManager<X>)>;

/// A function that is run when a mousebind is invoked.
///
/// An additional Point is supplied to track the location of the pointer.
pub type MouseCallback<X> = Box<dyn FnMut(&mut WindowManager<X>, Point)>;

/// A set of keybinds that can be run by the the window manager.
///
/// It consists of two components: A keybind, and its associated
/// callback function. It accepts a mutable reference to a
/// WindowManager to run associated methods.
///
/// Clone is not implemented for this type since Callbacks are not Clone.
#[derive(Default, Debug)]
pub struct Keybinds<X>
where
    X: XConn,
{
    bindings: HashMap<Keybind, KeyCallback<X>>,
}

impl<X: XConn> Keybinds<X> {
    /// Creates a new Keybinds object.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Returns an iterator over the keybinds stored inside.
    pub fn keys(&self) -> impl Iterator<Item = &Keybind> {
        self.bindings.keys()
    }

    /// Inserts a new keybind-callback mapping.
    pub fn insert<F>(&mut self, kb: Keybind, cb: F)
    where
        F: FnMut(&mut WindowManager<X>) + 'static,
    {
        self.bindings.insert(kb, Box::new(cb));
    }

    /// Removes the callback associated with the given keybind.
    pub fn remove(&mut self, kb: &Keybind) -> Option<KeyCallback<X>> {
        self.bindings.remove(kb)
    }

    /// Gets a reference to the callback associated with the keybind.
    pub fn get(&self, kb: &Keybind) -> Option<&KeyCallback<X>> {
        self.bindings.get(kb)
    }

    /// Gets a mutable reference to the callback associated with the keybind.
    pub fn get_mut(&mut self, kb: &Keybind) -> Option<&mut KeyCallback<X>> {
        self.bindings.get_mut(kb)
    }
}

/// A set of mousebinds that can be run by the window manager.
///
/// Like Keybinds, it consists of a mousebind and its associated
/// callback function. It accepts a mutable reference to a WindowManager
/// and a [Point][1], which contains the current coordinates of the pointer.
/// This point is used internally by the WindowManager and should not appear
/// in the user-facing API.
///
/// Clone is not implemented for this type since Callbacks are not Clone.
///
/// [1]: crate::core::types::Point
#[derive(Default, Debug)]
pub struct Mousebinds<X>
where
    X: XConn,
{
    bindings: HashMap<Mousebind, MouseCallback<X>>,
}

impl<X: XConn> Mousebinds<X> {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &Mousebind> {
        self.bindings.keys()
    }

    pub fn insert<F>(&mut self, kb: Mousebind, cb: F)
    where
        F: FnMut(&mut WindowManager<X>, Point) + 'static,
    {
        self.bindings.insert(kb, Box::new(cb));
    }

    pub fn remove(&mut self, kb: &Mousebind) -> Option<MouseCallback<X>> {
        self.bindings.remove(kb)
    }

    pub fn get(&self, kb: &Mousebind) -> Option<&MouseCallback<X>> {
        self.bindings.get(kb)
    }

    pub fn get_mut(&mut self, kb: &Mousebind) -> Option<&mut MouseCallback<X>> {
        self.bindings.get_mut(kb)
    }
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
