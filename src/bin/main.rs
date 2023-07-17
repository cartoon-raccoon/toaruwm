//! This is placeholder code for the purpose of testing ToaruWM's
//! functionality. A lot of this will be hidden from the user end
//! through procedural macros, so as to provide a more user-friendly
//! interface to configuring ToaruWM via source code.
//!
//! The rest of the comments are a tour through the behind-the-scenes
//! of how ToaruWM is configured.
//!
#![allow(clippy::type_complexity)]

use std::error::Error;

// use flexi_logger::{
//     Logger,
//     LogSpecification,
// };
use tracing::Level;
use tracing_subscriber::{fmt as logger, fmt::format::FmtSpan};

use toaruwm::bindings::{
    mb, ButtonIndex as Idx, Keybinds, Keymap, ModKey, MouseEventKind::*, Mousebinds,
};
use toaruwm::types::{Cardinal::*, Direction::*};
use toaruwm::{hook, ToaruConfig, WindowManager};
use toaruwm::{ToaruWM, InitX11RB};

// convenience typedef
type Wm<'a> = &'a mut ToaruWM<InitX11RB>;

//* defining keybinds and associated WM actions
const KEYBINDS: &[(&str, fn(Wm))] = &[
    ("M-Return",  |wm| wm.run_external("alacritty", &[])),
    ("M-r",       |wm| wm.run_external("dmenu_run", &["-b"])),
    ("M-q",       |wm| wm.close_focused_window()),
    ("M-S-d",     |wm| wm.dump_internal_state()),
    ("M-S-q",     |wm| wm.quit()),

    ("M-k",       |wm| wm.cycle_focus(Forward)),
    ("M-j",       |wm| wm.cycle_focus(Backward)),

    ("M-S-Up",    |wm| wm.warp_window(5, Up)),
    ("M-S-Down",  |wm| wm.warp_window(5, Down)),
    ("M-S-Left",  |wm| wm.warp_window(5, Left)),
    ("M-S-Right", |wm| wm.warp_window(5, Right)),

    ("M-t",       |wm| wm.toggle_focused_state()),

    ("M-Left",    |wm| wm.cycle_workspace(Backward)),
    ("M-Right",   |wm| wm.cycle_workspace(Forward)),

    ("M-1",       |wm| wm.goto_workspace("1")),
    ("M-2",       |wm| wm.goto_workspace("2")),
    ("M-3",       |wm| wm.goto_workspace("3")),

    ("M-S-1",     |wm| wm.send_focused_to("1")),
    ("M-S-2",     |wm| wm.send_focused_to("2")),
    ("M-S-3",     |wm| wm.send_focused_to("3")),

    ("M-Tab",     |wm| wm.cycle_layout(Forward)),
];

pub fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // set up the logger
    logger::fmt()
        // only log enter and exit
        .with_span_events(FmtSpan::ACTIVE)
        // log all events up to TRACE
        .with_max_level(Level::TRACE)
        // don't use timestamps
        .without_time()
        // don't show source filename
        .with_file(false)
        // don't show source code line
        .with_line_number(false)
        // register as global
        .try_init()?;

    //* 1: Setup X Connection and allocate new WM object
    let mut manager = toaruwm::x11rb_backed_wm(ToaruConfig::default())?;

    //* 2: Read/setup config
    // if using as a library, declare config here
    // else use a Config type to read a config file

    let keymap = Keymap::new()?;

    // adding keybinds
    let mut keybinds = Keybinds::new();
    for (kb, cb) in KEYBINDS {
        keybinds.insert(keymap.parse_keybinding(kb)?, Box::new(cb));
    }

    // adding mousebinds
    let mut mousebinds = Mousebinds::new();
    mousebinds.insert(
        mb(vec![ModKey::Meta], Idx::Left, Motion),
        Box::new(|wm: Wm, pt| wm.move_window_ptr(pt)),
    );
    mousebinds.insert(
        mb(vec![ModKey::Meta], Idx::Right, Motion),
        Box::new(|wm: Wm, pt| wm.resize_window_ptr(pt)),
    );

    //* create a hook if you want
    let test_hook = hook!(|wm| {
        wm.dump_internal_state();
        println!("hello from a hook!");
    });

    //* 3: Register the WM as a client with the X server
    //*    and initialise internal state
    //* a: Grab keys and mousebinds
    manager.register(vec![test_hook]);
    manager.grab_bindings(&keybinds, &mousebinds)?;

    //* 4: We're good to go!
    manager.run(keybinds, mousebinds)?;

    Ok(())
}
