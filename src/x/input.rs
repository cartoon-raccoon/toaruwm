//! Type definitions for various input types defined
//! by the X server protocol.

use std::ops::{BitAnd, BitOr};

use bitflags::bitflags;

use crate::{
    x::core::BitMask, 
    keybinds::{
        ModKey, ButtonIndex,
        Mousebind,
    },
};

//* Re-exports
pub mod keysym {
    pub type KeySym = u32;
    pub use x11::keysym::*;
}

// Grab NumLock separately and filter it out when receiving events
pub(crate) const MODIFIERS: &[ModMask] = &[
    ModMask::empty(),
    ModMask::MOD2,
];

/// A keycode as received from the X server.
pub type KeyCode = u8;

/// A type representing the type of mouse event sent by the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseEventKind {
    Motion,
    Press,
    Release,
}

bitflags!{

/// Bitmask representing one or a combination of modifier keys.
/// 
/// See definition in the X Server Protocol.
pub struct ModMask: u16 {
    /// The Shift key.
    const SHIFT   = 1 << 0;
    /// The Capslock key.
    const LOCK    = 1 << 1;
    /// The Control key.
    const CONTROL = 1 << 2;
    /// The Alt key.
    const MOD1    = 1 << 3;
    /// The Numlock key.
    const MOD2    = 1 << 4;
    const MOD3    = 1 << 5;
    /// The Super/Meta/Windows key.
    const MOD4    = 1 << 6;
    const MOD5    = 1 << 7;
}

/// Bitmask representing one or a combination of mouse buttons.
pub struct ButtonMask: u16 {
    const M1 = 1 << 8;
    const M2 = 1 << 9;
    const M3 = 1 << 10;
    const M4 = 1 << 11;
    const M5 = 1 << 12;
}

/// Union of ModMask and ButtonMask.
/// 
/// This struct conbines the individual definitions of
/// ModMask and ButtonMask into one single bitmask.
///
/// Bitwise operations are defined between ModMask and ButtonMask
/// that yield a KeyButMask, and bitwise operations
/// can be performed between a KeyButMask and ModMask or ButtonMask.
/// 
/// So, for example, this operation works:
/// 
/// ```rust
/// use toaruwm::x::input::{ModMask, ButtonMask, KeyButMask};
/// 
/// let shift = ModMask::SHIFT;
/// let mousebut = ButtonMask::M1;
/// 
/// let combined: KeyButMask = shift | mousebut;
/// 
/// assert_eq!(combined, (KeyButMask::SHIFT|KeyButMask::M1));
/// ```
pub struct KeyButMask: u16 {
    const SHIFT   = ModMask::SHIFT.bits();
    const LOCK    = ModMask::LOCK.bits();
    const CONTROL = ModMask::CONTROL.bits();
    const MOD1    = ModMask::MOD1.bits();
    const MOD2    = ModMask::MOD2.bits();
    const MOD3    = ModMask::MOD3.bits();
    const MOD4    = ModMask::MOD4.bits();
    const MOD5    = ModMask::MOD5.bits();
    const M1      = ButtonMask::M1.bits();
    const M2      = ButtonMask::M2.bits();
    const M3      = ButtonMask::M3.bits();
    const M4      = ButtonMask::M4.bits();
    const M5      = ButtonMask::M5.bits();
    const ANY = 1 << 15;
}

}

impl From<ModKey> for ModMask {
    fn from(from: ModKey) -> ModMask {
        match from {
            ModKey::Alt => ModMask::MOD1,
            ModKey::Shift => ModMask::SHIFT,
            ModKey::Ctrl => ModMask::CONTROL,
            ModKey::Meta => ModMask::MOD4,
        }
    }
}

impl From<ButtonIndex> for ButtonMask {
    fn from(from: ButtonIndex) -> ButtonMask {
        match from {
            ButtonIndex::Left => ButtonMask::M1,
            ButtonIndex::Middle => ButtonMask::M2,
            ButtonIndex::Right => ButtonMask::M3,
            ButtonIndex::Button4 => ButtonMask::M4,
            ButtonIndex::Button5 => ButtonMask::M5,
        }
    }
}

//? FIXME: TEST THIS!!!
impl KeyButMask {
    /// Extracts the modmask portion of the bits from `self`.
    pub fn modmask(self) -> ModMask {
        ModMask::from_bits_truncate(
            self.intersection(
                Self::from_bits_truncate(ModMask::all().bits())
            ).bits()
        )
    }

    /// Extracts the buttonmask portion of the bits from `self`.
    pub fn buttonmask(self) -> ButtonMask {
        ButtonMask::from_bits_truncate(
            self.intersection(
                Self::from_bits_truncate(ButtonMask::all().bits())
            ).bits()
        )
    }
}

impl BitMask for ModMask {}
impl BitMask for ButtonMask {}
impl BitMask for KeyButMask {}

macro_rules! _impl_bitwise {
    (And: $lhs:ty, $rhs:ty => $output:ty) => {
        impl BitAnd<$rhs> for $lhs {
            type Output = $output;
            fn bitand(self, rhs: $rhs) -> $output {
                <$output>::from_bits_truncate(self.bits() & rhs.bits())
            }
        }
    };
    (Or: $lhs:ty, $rhs:ty => $output:ty) => {
        impl BitOr<$rhs> for $lhs {
            type Output = $output;
            fn bitor(self, rhs: $rhs) -> $output {
                <$output>::from_bits_truncate(self.bits() | rhs.bits())
            }
        }
    }
}

// ops between KeyButMask and ModMask yielding KeyButMask
_impl_bitwise!(And: KeyButMask, ModMask => KeyButMask);
_impl_bitwise!(Or: KeyButMask, ModMask => KeyButMask);
_impl_bitwise!(And: ModMask, KeyButMask => KeyButMask);
_impl_bitwise!(Or: ModMask, KeyButMask => KeyButMask);

// ops between KeyButMask and ButtonMask yielding KeyButMask
_impl_bitwise!(And: KeyButMask, ButtonMask => KeyButMask);
_impl_bitwise!(Or: KeyButMask, ButtonMask => KeyButMask);
_impl_bitwise!(And: ButtonMask, KeyButMask => KeyButMask);
_impl_bitwise!(Or: ButtonMask, KeyButMask => KeyButMask);

// ops between ModMask and ButtonMask yielding KeyButMask
_impl_bitwise!(And: ModMask, ButtonMask => KeyButMask);
_impl_bitwise!(Or: ModMask, ButtonMask => KeyButMask);
_impl_bitwise!(And: ButtonMask, ModMask => KeyButMask);
_impl_bitwise!(Or: ButtonMask, ModMask => KeyButMask);

impl Mousebind {
    /// Express the modifier mask as an generic type.
    pub(crate) fn modmask<T>(&self) -> T
    where
        T: BitMask + From<ModMask> {
        self.modmask.into()
    }
}

impl ModKey {
    /// Tests if a
    pub(crate) fn was_held<M: Into<ModMask>>(&self, state: M) -> bool {
        let state = state.into();
        match *self {
            Self::Ctrl  => state.contains(ModMask::CONTROL),
            Self::Alt   => state.contains(ModMask::MOD1),
            Self::Shift => state.contains(ModMask::SHIFT),
            Self::Meta  => state.contains(ModMask::MOD4)
        }
    }
}
