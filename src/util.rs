use xcb::x::EventMask;

pub const FOCUSED_COL: u32 = 0xdddddd;
pub const UNFOCUSED_COL: u32 = 0x555555;
pub const URGENT_COL: u32 = 0xdd0000;

// Root window mouse button event mask
pub const ROOT_BUTTON_GRAB_MASK: EventMask = 
    EventMask::BUTTON_PRESS.union(EventMask::BUTTON_RELEASE);

// Root window pointer event mask
pub const ROOT_POINTER_GRAB_MASK: EventMask = 
    EventMask::BUTTON_RELEASE.union(EventMask::BUTTON_MOTION);
