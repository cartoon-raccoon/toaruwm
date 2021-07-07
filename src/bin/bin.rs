//! This is placeholder code for the purpose of testing ToaruWM's 
//! functionality. A lot of this will be hidden from the user end 
//! through procedural macros, so as to provide a more user-friendly 
//! interface to configuring ToaruWM via source code.
//! 
//! The rest of the comments are a tour through the behind-the-scenes 
//! of how ToaruWM is configured.

use std::error::Error;

use toaruwm::xcb_backed_wm;
use toaruwm::keybinds::{
    ModKey,
    Keymap,
    Keybinds,
    Mousebinds,
    mb,
    MouseEventKind::*,
    ButtonIndex as Idx,
};
use toaruwm::types::Cardinal::*;
use toaruwm::x::xcb::XCBConn;
use toaruwm::WindowManager;

use std::collections::HashMap;

// convenience typedef
type Wm<'a> = &'a mut WindowManager<XCBConn>;

//* defining keybinds and associated WM actions
const KEYBINDS: &[(&str, fn(Wm))] = &[
    ("M-Return", |wm| wm.run_external("alacritty", &[])),
    ("M-r",      |wm| wm.run_external("dmenu_run", &["-b"])),
    ("M-q",      |wm| wm.close_focused_window()),
    ("M-S-d",    |wm| wm.dump_internal_state()),
    ("M-S-q",    |wm| wm.quit()),
    ("M-S-Up",   |wm| wm.warp_window(5, Up)),
    ("M-S-Down", |wm| wm.warp_window(5, Down)),
    ("M-S-Left", |wm| wm.warp_window(5, Left)),
    ("M-S-Right",|wm| wm.warp_window(5, Right)),
    ("M-1",      |wm| wm.goto_workspace("1")),
    ("M-2",      |wm| wm.goto_workspace("2")),
];

pub fn main() -> Result<(), Box<dyn Error>> {
    //* 1: Setup X Connection and allocate new WM object
    let mut wm = xcb_backed_wm()?;

    //* 2: Read/setup config
    // if using as a library, declare config here
    // else use a Config type to read a config file

    let keymap = Keymap::new()?;

    // adding keybinds
    let mut keybinds: Keybinds<XCBConn> = HashMap::new();
    for (kb, cb) in KEYBINDS {
        keybinds.insert(keymap.parse_keybinding(kb)?, Box::new(cb));
    }

    // adding mousebinds
    let mut mousebinds: Mousebinds<XCBConn> = HashMap::new();
    mousebinds.insert(
        mb(vec![ModKey::Meta], Idx::Left, Motion), Box::new(|wm: Wm, pt| wm.move_window_ptr(pt))
    );
    mousebinds.insert(
        mb(vec![ModKey::Meta], Idx::Right, Motion), Box::new(|wm: Wm, pt| wm.resize_window_ptr(pt))
    );

    //* 3: Register the WM as a client with the X server
    //*    and initialise internal state
    //* a: Grab keys and mousebinds
    wm.register(Vec::new() /* pass config in here */);

    //* 4: Run the WM
    wm.grab_and_run(mousebinds, keybinds)?;

    Ok(())
}