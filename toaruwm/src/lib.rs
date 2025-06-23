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
//! dwm or Qtile. Its default tiling layout maintains a main window
//! on the left of the screen, while other windows are stacked on the
//! side of the main window. Users can also design their own layouts
//! and switch between each layout on the fly.
//!
//! Like Qtile and dwm, ToaruWM also maintains a number of workspaces
//! that the user can switch between using bindings. Each workspace
//! has its own set of layouts that can be hotswapped.
//!
//! ToaruWM also provides the ability to run arbitrary commands and
//! code at almost any point in the runtime of the window manager,
//! through hooks. This means that the user can run various commands
//! such as an autostart script, either by invoking a shell script
//! or directly in the window manager. Hooks can also be triggered
//! on various events, such as mapping/unmapping of a certain window.
//!
//! ToaruWM was designed to be all about choices. You write it the way
//! you want, from the ground up; what we do is provide the tools
//! to make it easier for you to do so.
//!
//! ## Usage
//!
//! Note that this crate, as it exists on Crates.io, is not a binary
//! that you can download and immediately run; you will have to create
//! a separate Rust project and pull this crate as a dependency, and
//! write a Rust program that ties everything inside this crate together
//! the way you want it. As such, you will need a working knowledge
//! of Rust, of which the [book](https://doc.rust-lang.org/book/) will
//! provide more than enough for you to get something up and running.
//!
//! The core of this crate is the central [`WindowManager`] struct;
//! it is the entry point to this crate, and everything else in this
//! crate is built around it. To start exploring this crate, reading
//! its documentation is the best place to start.
//!
//! That being said, a basic window manager built with ToaruWM has
//! the following general structure:
//!
//! ```no_run
//!# use toaruwm::WindowManager;
//!# use toaruwm::{ToaruWM, InitX11RB};
//!
//!# // convenience typedef
//! use toaruwm::{
//!     ToaruConfig,
//!     x11rb_backed_wm, hook
//! };
//! use toaruwm::bindings::{
//!     mb, ButtonIndex as Idx,
//!     Keymap, Keybinds, Mousebinds,
//!     ModKey, MouseEventKind::*,
//! };
//!
//! type Wm<'a> = &'a mut ToaruWM<InitX11RB>;
//!
//! //todo: hide all this behind a declarative macro
//! // defining keybinds and associated WM actions
//! const KEYBINDS: &[(&str, fn(Wm))] = &[
//!     ("M-q", |wm| wm.close_focused_window()),
//!     ("M-S-q", |wm| wm.quit()),
//! ];
//!
//! //* 1: Setup X Connection and allocate new WM object
//! let mut wm = x11rb_backed_wm(ToaruConfig::default()).unwrap();
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
//! ToaruWM core has internal support for widgets and extensions through
//! the [`Widget`](widget::Widget) trait.
//!
//! Additionally, the ToaruWM ecosystem takes the same approach as QTile:
//! everything _and_ the kitchen sink. A number of extensions and add-ons
//! such as bars, widgets, and configuration options will be provided
//! through the planned `toarulib` crate, which will contain
//! many different additional widgets that you can add you your own
//! personal configuration.
//!
//! Of course, you are still free to use your own bars such as Polybar:
//! ToaruWM is planned to have support for [EWMH], which are what
//! makes window managers aware of things like bars and fullscreen,
//! and account for them accordingly.
//!
//! ## Compliance
//!
//! ToaruWM is (planned to be) mostly compliant with EWMH, and
//! with most sections of the [ICCCM], particularly the ones that
//! were deemed most important for interoperability with various
//! X clients, such as notification daemons, pop-up windows,
//! full-screen clients, etc.
//!
//! Important to note is that this project does not, and will _never_
//! have, full compliance with ICCCM, partly because parts of ICCCM
//! have been superseded by EWMH, and also because other parts of ICCCM
//! are just [not worth implementing][1].
//!
//! For the full details on compliance, see the `COMPLIANCE` file
//! in this project's git repository.
//!
//! [EWMH]: https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints
//! [ICCCM]: https://en.wikipedia.org/wiki/Inter-Client_Communication_Conventions_Manual
//! [1]: http://www.call-with-current-continuation.org/rants/icccm.txt

#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs
)]

#[macro_use]
extern crate bitflags;

pub mod bindings;
pub mod core;
pub mod layouts;
pub mod manager;
pub mod toarulib;
pub mod platform;
pub mod util;

pub(crate) use util::log;

/// Modules that Toaru is tightly integrated with, re-exported for convenience.
pub mod reexports {
    pub use smithay;
    pub use smithay::reexports::calloop;
}

pub use crate::core::types;
#[doc(inline)]
pub use crate::manager::{Config, ToaruConfig, Toaru};
#[doc(inline)]
pub use crate::platform::x11::core::XConn;
#[doc(inline)]
pub use crate::platform::x11::{x11rb::X11RBConn, xcb::XCBConn};

pub use crate::platform::{Platform};

use crate::bindings::BindingError;
use crate::manager::state::{RuntimeConfig, WmConfig};
use crate::platform::x11::{Initialized};

use std::io;

/// Convenience type definition for a WindowManager
/// using a WmConfig as its RuntimeConfig.
pub type ToaruWM<X> = Toaru<X, WmConfig>;

/// Convenience type definition for an Initialized
/// XCBConn.
pub type InitXCB = XCBConn<Initialized>;

/// Convenience type definition for an Initialized
/// X11RBConn.
pub type InitX11RB = X11RBConn<Initialized>;

/// Convenience function for creating an `xcb`-backed `WindowManager`.
// pub fn xcb_backed_wm(config: ToaruConfig) -> Result<ToaruWM<InitXCB>> {
//     let conn = XCBConn::connect()?;
//     let conn = conn.init()?;

//     let wm = Toaru::new(conn, config)?;

//     Ok(wm)
// }

/// Convenience function for creating an `x11rb`-backed `WindowManager`.
// pub fn x11rb_backed_wm(config: ToaruConfig) -> Result<ToaruWM<InitX11RB>> {
//     let conn = X11RBConn::connect()?;
//     let conn = conn.init()?;

//     let wm = Toaru::new(conn, config)?;

//     Ok(wm)
// }

use thiserror::Error;

/// Everything that could possibly go wrong while Toaru is running.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ToaruError<P: Platform> {
    /// An error with the underlying X connection.
    #[error(transparent)]
    BackendError(P::Error),

    /// Unable to spawn process.
    #[error("Error while running program: {0}")]
    SpawnProc(String),

    /// An error occurred while parsing keybinds.
    #[error(transparent)]
    Bindings(BindingError),

    /// Unable to convert external data into an internal Toaru datatype.
    #[error("Could not convert external data type for internal use")]
    ConversionError,

    /// Received a reference to a client not tracked by ToaruWM.
    #[error("Unknown client {0:?}")]
    UnknownClient(P::Client),

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

//todo: example
/// Quickly construct a ToaruError.
#[macro_export]
macro_rules! toaruerr {
    // XConnError
    (xconn: $t:expr) => {
        ToaruError::XConnError($t)
    };
    // SpawnProc
    (spawn: $t:expr) => {
        ToaruError::SpawnProc($t)
    };
    // ParseKeybind
    (bindings: $t:expr) => {
        ToaruError::Bindings($t)
    };
    // ConversionError
    (converr: $t:expr) => {
        ToaruError::ConversionError
    };
    // UnknownClient
    (unknowncl: $t:expr) => {
        ToaruError::UnknownClient($t)
    };
    // UnknownWorkspace
    (unknownws: $t:expr) => {
        ToaruError::UnknownWorkspace($t)
    };
    // InvalidPoint
    (invalidpt: $x:expr, $y:expr) => {
        ToaruError::InvalidPoint($x, $y)
    };
    // LayoutConflict
    (layoutcf: $t:expr) => {
        ToaruError::LayoutConflict($t)
    };
    // InvalidConfig
    (invalidcfg: $t:expr) => {
        ToaruError::InvalidConfig($t)
    };
    // OtherError
    (other: $t:expr) => {
        ToaruError::OtherError($t)
    };
}

impl<P: Platform> From<io::Error> for ToaruError<P> {
    fn from(e: io::Error) -> ToaruError<P> {
        ToaruError::SpawnProc(e.to_string())
    }
}

/// The general result type used by ToaruWM.
pub type Result<T, P> = ::core::result::Result<T, ToaruError<P>>;

use crate::manager::ToaruState;
/// An error handler that can be used to handle an error type.
///
/// Typically this would be a standard logging function that writes
/// to a file or stdout, but it can be anything.
pub trait ErrorHandler<P, C>
where
    P: Platform,
    C: RuntimeConfig,
{
    /// Calls the error handler.
    fn call(&self, state: ToaruState<'_, P, C>, err: ToaruError<P>);
}
