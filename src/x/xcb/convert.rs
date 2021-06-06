use crate::keybinds::{
    ButtonMask,
    ButtonIndex,
    ModKey,
};

impl From<ButtonMask> for xcb::ButtonMask {
    fn from(from: ButtonMask) -> xcb::ButtonMask {
        use ButtonMask::*;

        match from {
            Left => xcb::BUTTON_MASK_1,
            Middle => xcb::BUTTON_MASK_2,
            Right => xcb::BUTTON_MASK_3,
            Button4 => xcb::BUTTON_MASK_4,
            Button5 => xcb::BUTTON_MASK_5,
        }
    }
}

impl From<ButtonIndex> for u8 {
    fn from(from: ButtonIndex) -> u8 {
        use ButtonIndex::*;

        match from {
            Left => xcb::BUTTON_INDEX_1 as u8,
            Middle => xcb::BUTTON_INDEX_2 as u8,
            Right => xcb::BUTTON_INDEX_3 as u8,
            Button4 => xcb::BUTTON_INDEX_4 as u8,
            Button5 => xcb::BUTTON_INDEX_5 as u8,
        }
    }
}

impl From<ModKey> for u16 {
    fn from(from: ModKey) -> u16 {
        use ModKey::*;

        match from {
            Ctrl => xcb::MOD_MASK_CONTROL as u16,
            Alt => xcb::MOD_MASK_1 as u16,
            Shift => xcb::MOD_MASK_SHIFT as u16,
            Meta => xcb::MOD_MASK_4 as u16,
        }
    }
}