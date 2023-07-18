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
//!# use toaruwm::x::X11RBConn;
//!# use toaruwm::WindowManager;
//!# use toaruwm::x::status::Initialized;
//!
//!# // convenience typedef
//!# type Wm<'a> = &'a mut WindowManager<X11RBConn<Initialized>>;
//! use toaruwm::{
//!     Config,
//!     x11rb_backed_wm, hook
//! };
//! use toaruwm::bindings::{
//!     mb, ButtonIndex as Idx,
//!     Keymap, Keybinds, Mousebinds,
//!     ModKey, MouseEventKind::*,
//! };
//!
//! //todo: hide all this behind a declarative macro
//! // defining keybinds and associated WM actions
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
//! wm.grab_bindings(&keybinds, &mousebinds).unwrap();
//!
//! //* 4: We're good to go!
//! wm.run(keybinds, mousebinds).unwrap();
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

#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs
)]

#[macro_use]
extern crate bitflags;

#[macro_use]
mod log;

pub mod core;
pub mod bindings;
pub mod layouts;
pub mod manager;
pub mod x;

pub use crate::core::types;
#[doc(inline)]
pub use crate::manager::{Config, ToaruConfig, WindowManager};
#[doc(inline)]
pub use crate::x::core::XConn;
#[doc(inline)]
pub use crate::x::{x11rb::X11RBConn, xcb::XCBConn};

use crate::manager::state::{WmConfig, RuntimeConfig};
use crate::x::Initialized;

use std::io;
use std::num::ParseIntError;

/// Convenience type definition for a WindowManager
/// using a WmConfig as its RuntimeConfig.
pub type ToaruWM<X> = WindowManager<X, WmConfig>;

/// Convenience type definition for an Initialized
/// XCBConn.
pub type InitXCB = XCBConn<Initialized>;

/// Convenience type definition for an Initialized
/// X11RBConn.
pub type InitX11RB = X11RBConn<Initialized>;

/// Convenience function for creating an `xcb`-backed `WindowManager`.
pub fn xcb_backed_wm(config: ToaruConfig) -> Result<ToaruWM<InitXCB>> {
    let conn = XCBConn::connect()?;
    let conn = conn.init()?;

    let wm = WindowManager::new(conn, config)?;

    Ok(wm)
}

/// Convenience function for creating an `x11rb`-backed `WindowManager`.
pub fn x11rb_backed_wm(config: ToaruConfig) -> Result<ToaruWM<InitX11RB>> {
    let conn = X11RBConn::connect()?;
    let conn = conn.init()?;

    let wm = WindowManager::new(conn, config)?;

    Ok(wm)
}

use crate::x::core::{XError, XWindowID};
use thiserror::Error;

/// Everything that could possibly go wrong while ToaruWM is running.
#[non_exhaustive]
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

    /// An error occurred when parsing a keybind specification.
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

    /// A name conflict in the given set of layouts.
    #[error("Layout name conflict: {0}")]
    LayoutConflict(String),

    /// One or more configuration invariants was not upheld.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// An error not covered by ToaruWM.
    #[error("Error: {0}")]
    OtherError(String),
}

//todo
/// Quickly construct a ToaruError.
#[macro_export]
macro_rules! error {
    () => {}
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

use crate::manager::WmState;
/// An error handler that can be used to handle an error type.
///
/// Typically this would be a standard logging function that writes
/// to a file or stdout, but it can be anything.
pub trait ErrorHandler<X, C>
where
    X: XConn,
    C: RuntimeConfig
{
    /// Calls the error handler.
    fn call(&self, state: WmState<'_, X, C>, err: ToaruError);
}
