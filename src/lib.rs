//! # ToaruWM - A certain X WM
//! 
//! ToaruWM is a crate that gives you the tools you need to design
//! and build your very own X window manager.
//! It provides useful types and interfaces that you can use to put
//! a window manager together, in any way you want. You can design
//! a full-blown window manager configured with a completely different
//! language (a la AwesomeWM), or a tiny, minimal window manager
//! that is configured within the source code and requires a recompile
//! (like dwm) each time you change the config.
//! 
//! ## Design
//! 
//! ToaruWM was designed in the style of tiling window managers, like
//! dwm or Qtile. It maintains a main window on the left of the screen,
//! while other windows are stacked on the side of the main window.
//! Users can also design their own layouts and switch between each
//! layout on the fly.
//! 
//! Like Qtile and dwm, ToaruWM also maintains a number of workspaces
//! that the user can switch between using bindings. Each workspace
//! has its own layout that can be hotswapped, as mentioned above.
//! 
//! ToaruWM also provides the ability to run arbitrary commands and
//! code at almost any point in the runtime of the window manager,
//! through hooks. This means that the user can run various commands
//! such as an autostart script, either by invoking a shell script
//! or directly in the window manager. Hooks can also be triggered
//! on various events, such as mapping/unmapping of a certain window.
//! 
//! ## Usage
//! 
//! Note that this crate, as it exists on Crates.io, is not a binary 
//! that you can download and immediately run; you will have to create
//! a separate Rust project and pull this crate as a dependency, and
//! write a Rust program that ties everything inside this crate together
//! the way you want it. As such, you will need a working knowledge
//! of Rust, which the [book](https://doc.rust-lang.org/book/) will
//! provide more than enough for you to get something up and running.
//! 
//! That being said, a basic window manager built with ToaruWM has
//! the following general structure:
//! 
//! ```no_run
//! use toaruwm::x::X11RBConn;
//! use toaruwm::{x11rb_backed_wm, hook};
//! use toaruwm::{WindowManager, Config};
//! use toaruwm::keybinds::{
//!     mb, ButtonIndex as Idx,
//!     Keymap, Keybinds, Mousebinds, 
//!     ModKey, MouseEventKind::*,
//! };
//! 
//! // convenience typedef
//! type Wm<'a> = &'a mut WindowManager<X11RBConn>;
//!
//! //* defining keybinds and associated WM actions
//! const KEYBINDS: &[(&str, fn(Wm))] = &[
//!     ("M-q", |wm| wm.close_focused_window()),
//!     ("M-S-q", |wm| wm.quit()),
//! ];
//! 
//! //* 1: Setup X Connection and allocate new WM object
//! let mut wm = x11rb_backed_wm(Config::default()).unwrap();
//!
//! //* 2: Read/setup config
//! // if using as a library, declare config here
//! // else use a Config type to read a config file
//!
//! let keymap = Keymap::new().unwrap();
//!
//! // adding keybinds
//! let mut keybinds = Keybinds::new();
//! for (kb, cb) in KEYBINDS {
//!     keybinds.insert(
//!         keymap.parse_keybinding(kb).unwrap(), 
//!         Box::new(cb)
//!     );
//! }
//!
//! // adding mousebinds
//! let mut mousebinds = Mousebinds::new();
//! mousebinds.insert(
//!     mb(vec![ModKey::Meta], Idx::Left, Motion),
//!     Box::new(|wm: Wm, pt| wm.move_window_ptr(pt)),
//! );
//! mousebinds.insert(
//!     mb(vec![ModKey::Meta], Idx::Right, Motion),
//!     Box::new(|wm: Wm, pt| wm.resize_window_ptr(pt)),
//! );
//!
//! //* create a hook if you want
//! let test_hook = hook!(|wm| {
//!     wm.dump_internal_state();
//!     println!("hello from a hook!");
//! });
//!
//! //* 3: Register the WM as a client with the X server
//! //*    and initialise internal state
//! //* a: Grab keys and mousebinds
//! wm.register(vec![test_hook]);
//! wm.grab_bindings(&mousebinds, &keybinds).unwrap();
//!
//! //* 4: We're good to go!
//! wm.run(mousebinds, keybinds).unwrap();
//! 
//! ```
//! 
//! ## Extensions and Add-Ons
//! 
//! ToaruWM provides a number of extensions similar to how QTile
//! does it. PLANNED
//! 
//! ## Compliance
//! 
//! ToaruWM is compliant with most sections of the ICCCM and EWMH,
//! particularly the ones that were deemed most important for
//! interoperability with various X clients, such as notification
//! daemons, pop-up windows, full-screen clients, etc.
//! 
//! For the full details on compliance, see the `COMPLIANCE` file
//! in this project's git repository.

#[macro_use]
extern crate bitflags;

#[macro_use]
mod log;

/// Core data types not used by ToaruWM.
pub mod core;
/// Types for parsing and creating key and mouse bindings.
pub mod keybinds;
/// Types and traits for defining and generating window layouts.
pub mod layouts;
/// The window manager itself, and associated modules.
pub mod manager;
/// Types and traits providing a unified interface with the X server.
pub mod x;

pub(crate) mod util;

pub use crate::core::types;
pub use crate::manager::{WindowManager, Config};
pub use crate::x::core::Result as XResult;
pub use crate::x::core::XConn;
pub use crate::x::{x11rb::X11RBConn, xcb::XCBConn};

use std::io;
use std::num::ParseIntError;
use std::ops::FnMut;

/// Convenience function for creating a XCB-backed WindowManager.
pub fn xcb_backed_wm(config: Config) -> XResult<WindowManager<XCBConn>> {
    let mut conn = XCBConn::connect()?;
    conn.init()?;

    let wm = WindowManager::new(conn, config);

    Ok(wm)
}

/// Convenience function for creating a XCB-backed WindowManager.
pub fn x11rb_backed_wm(config: Config) -> XResult<WindowManager<X11RBConn>> {
    let mut conn = X11RBConn::connect()?;
    conn.init()?;

    let wm = WindowManager::new(conn, config);

    Ok(wm)
}

use crate::x::core::{XError, XWindowID};
use thiserror::Error;

/// Everything that could possibly go wrong while ToaruWM is running.
#[derive(Debug, Error, Clone)]
pub enum ToaruError {
    /// An error with the underlying X connection.
    #[error(transparent)]
    XConnError(XError),

    /// Unable to spawn process.
    #[error("Error while running program: {0}")]
    SpawnProc(String),

    /// Unable to parse an X data type into a type known to ToaruWM.
    #[error("Could not parse X data type from integer")]
    ParseInt,

    #[error("Could not parse keybind \"{0}\"")]
    ParseKeybind(String),

    /// Unable to convert external data into an internal Toaru datatype.
    #[error("Could not convert external data type for internal use")]
    ConversionError,

    /// Received a reference to a client not tracked by ToaruWM.
    #[error("Unknown client {0}")]
    UnknownClient(XWindowID),

    /// An request to switch to a workspace unknown to ToaruWM.
    #[error("Unknown workspace {0}")]
    UnknownWorkspace(String),

    /// An invalid point on the root window.
    #[error("Invalid point ({0}, {1})")]
    InvalidPoint(i32, i32),

    /// An error not covered by ToaruWM.
    #[error("Error: {0}")]
    OtherError(String),
}

impl From<XError> for ToaruError {
    fn from(e: XError) -> ToaruError {
        ToaruError::XConnError(e)
    }
}

impl From<io::Error> for ToaruError {
    fn from(e: io::Error) -> ToaruError {
        ToaruError::SpawnProc(e.to_string())
    }
}

impl From<ParseIntError> for ToaruError {
    fn from(_: ParseIntError) -> ToaruError {
        ToaruError::ParseInt
    }
}

/// The general result type used by ToaruWM.
pub type Result<T> = ::core::result::Result<T, ToaruError>;

/// An error handler that can be used to handle an error type.
///
/// Typically this would be a standard logging function that writes
/// to a file or stdout, but it can be anything.
pub type ErrorHandler = Box<dyn FnMut(ToaruError)>;
