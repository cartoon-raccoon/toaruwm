use crate::x::xproto;

pub const FOCUSED_COL: u32 = 0xdddddd;
pub const UNFOCUSED_COL: u32 = 0x555555;
pub const URGENT_COL: u32 = 0xdd0000;

pub const ROOT_ATTRS: [(u32, u32); 1] = [
    (
        xproto::CW_EVENT_MASK, 
        xproto::EVENT_MASK_SUBSTRUCTURE_REDIRECT | 
        xproto::EVENT_MASK_STRUCTURE_NOTIFY |
        xproto::EVENT_MASK_PROPERTY_CHANGE
    )
];

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

pub fn disable_events() -> [(u32, u32); 1] {
    //debug!("VALUES: attributes no events");
    [(xproto::CW_EVENT_MASK, xproto::EVENT_MASK_NO_EVENT)]
}

pub fn child_events() -> [(u32, u32); 1] {
    //debug!("VALUES: attributes child events");
    [(xproto::CW_EVENT_MASK,
        xproto::EVENT_MASK_ENTER_WINDOW|    // -> Self EnterNotify events
        xproto::EVENT_MASK_STRUCTURE_NOTIFY // -> Self CirculateNotify, ConfigureNotify, DestroyNotify, GravityNotify, MapNotify, ReparentNotify, UnmapNotify events
    )]
}

pub fn configure_move(x: u32, y: u32) -> [(u16, u32); 2] {
    //debug!("VALUES: configure move");
    [(xproto::CONFIG_WINDOW_X as u16, x), (xproto::CONFIG_WINDOW_Y as u16, y)]
}

pub fn configure_resize(width: u32, height: u32) -> [(u16, u32); 2] {
    //debug!("VALUES: configure resize");
    [(xproto::CONFIG_WINDOW_WIDTH as u16, width), (xproto::CONFIG_WINDOW_HEIGHT as u16, height)]
}

pub fn stack_above() -> [(u16, u32); 1] {
    //debug!("VALUES: configure stack above sibling {}", window_id);
    [
        (xproto::CONFIG_WINDOW_STACK_MODE as u16, xproto::STACK_MODE_ABOVE),
        //(xproto::CONFIG_WINDOW_SIBLING as u16, window_id),
    ]
}