use crate::x::xproto;

pub const FOCUSED_COL: u32 = 0xdddddd;
pub const UNFOCUSED_COL: u32 = 0x555555;
pub const URGENT_COL: u32 = 0xdd0000;

// Root window mouse button event mask
pub const ROOT_BUTTON_GRAB_MASK: xproto::ButtonMask = 
    xproto::EVENT_MASK_BUTTON_PRESS | xproto::EVENT_MASK_BUTTON_RELEASE;

// Root window pointer event mask
pub const ROOT_POINTER_GRAB_MASK: xproto::EventMask = 
    xproto::EVENT_MASK_BUTTON_RELEASE | xproto::EVENT_MASK_BUTTON_MOTION;

pub fn cursor_attrs(cursor_id: u32) -> [(u32, u32); 1] {
    //debug!("Getting cursor attrs for cursor {}", cursor_id);
    [(xproto::CW_CURSOR, cursor_id)]
}
