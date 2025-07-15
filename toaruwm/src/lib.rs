//! # Toaru - A certain desktop
//!
//! Toaru is a crate that gives you the tools you need to design and build your very own graphical desktop, 
//! without having to dig into the complex plumbing of how it actually works, though it gives you the ability
//! to dig into it, and replace individual parts if you want. It provides useful types and interfaces that 
//! you can use to put a desktop together, in any way you want, on X11 or Wayland. You  can design a full-blown 
//! desktop configured with a completely different language (a la AwesomeWM), or a tiny, minimal window 
//! manager that is configured within the source code and requires a recompile (like dwm) each time you 
//! change the config.
//!
//! ## Design
//!
//! Toaru was designed around traits, and every major component of a complete Toaru implementation is coupled
//! together by traits. This allows you to implement exactly as much as you want, and whatever you want to
//! delegate to this crate, it can provide an implementation that will slot in nicely without any fuss.
//! 
//! ### Core Traits
//! 
//! Toaru's two core traits are the [`Manager`] and [`Platform`] traits, and everything in this crate ultimately
//! revolves around these two traits. The `Manager` trait defines the abstract window management functionality
//! such as window layout, while the `Platform` trait defines the functionality provided by the underlying
//! graphical platform, such as Wayland or the X Server.
//! 
//! For more information, consult their module-level documentation.
//! 
//! ### Event Loops and Tracking State
//! 
//! A program that implements a graphical desktop will invariably incorporate an event loop, as it
//! needs to receive and respond to various requests. As such, there is a lot of state of keep track of
//! in various places, and multiple places will need to own a reference to the same data, to make it
//! easier to maintain internal consistency.
//! 
//! As such, Toaru's various implementations make the assumption that all of `Platform`'s associated types
//! incorporate some form of reference counting for multiple ownership, such as using [`std::rc::Rc`] or its 
//! [atomic variant][4], using something like [`std::cell::RefCell`] for interior mutability. For example, 
//! the associated types used by the [`Wayland`] implementation use Smithay's implementations of [`Window`][5] 
//! and [`Output`][6], which are internally reference counted. If your code makes use of any of Toaru's types
//! anywhere, it should take this into account.
//! 
//! ### A Basic Example
//! 
//! A basic desktop built with Toaru that runs on Wayland has the following general structure:
//!
//! ```no_run
//! // todo
//! ```
//! 
//! ### Feature Flags
//! 
//! Toaru provides feature flags to reduce the amount of compiled code. // todo
//! 
//!
//! ## Usage
//!
//! Note that this crate, as it exists on Crates.io, is not a binary that you can download and immediately run;
//! you will have to create a separate Rust project and pull this crate as a dependency, and write a Rust program
//! that ties everything inside this crate together the way you want it. As such, you will need a working knowledge
//! of Rust, of which the [book](https://doc.rust-lang.org/book/) will provide more than enough for you to get
//! something up and running.
//! 
//! However, if you wish to just install the default implementation, you can install it here: // todo
//!
//! ## Extensions and Add-Ons
//!
//! Toaru core has internal support for widgets and extensions. The Toaru ecosystem takes the same approach as 
//! QTile: everything _and_ the kitchen sink. A number of extensions and add-ons such as bars, widgets, and 
//! configuration options are provided in the `toarulib` module, which will contain many different additional 
//! widgets that you can add you your own personal configuration.
//!
//! Of course, you are still free to use your own bars such as Polybar or Waybar: Toaru's `Platform` implementations
//! can recognize bars and allocate monitor space for them.
//!
//! ## Compliance
//! 
//! For the full details on compliance, see the `COMPLIANCE` file in this project's git repository.
//! 
//! ### X Window Protocol and Extensions
//!
//! Toaru is (planned to be) mostly compliant with [EWMH], and with most sections of the [ICCCM], particularly 
//! the ones that were deemed most important for interoperability with various X clients, such as notification daemons,
//! pop-up windows, full-screen clients, etc.
//!
//! Important to note is that this project does not, and will _never_ have, full compliance with ICCCM, partly because
//! parts of ICCCM have been superseded by EWMH, and also because other parts of ICCCM are just
//! [not worth implementing][1].
//! 
//! ### Wayland
//! 
//! Toaru is compliant with all core and stable protocols, and some unstable protocols. See the COMPLIANCE file for full
//! details.
//!
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

pub mod config;
pub mod bindings;
pub mod core;
pub mod layouts;
pub mod manager;
pub mod toarulib;
pub mod platform;
pub mod util;

/// Modules that Toaru is tightly integrated with, re-exported for convenience.
pub mod reexports {
    pub use smithay;
    pub use smithay::reexports::calloop;
    pub use smithay::reexports::wayland_server;
    pub use smithay::reexports::wayland_protocols;
}

pub use crate::core::types;
#[doc(inline)]
pub use crate::manager::{Toaru, Manager};
#[doc(inline)]
pub use crate::config::{ManagerConfig, ToaruManagerConfig};
#[doc(inline)]
pub use crate::platform::{Platform, Wayland};

use crate::platform::PlatformError;

use crate::bindings::BindingError;
use crate::config::RuntimeConfig;

use std::io;

use thiserror::Error;

/// Everything that could possibly go wrong while Toaru is running.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ToaruError {
    /// An error with the underlying X connection.
    #[error("platform error: {0}")]
    PlatformError(Box<dyn PlatformError>),

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
    UnknownClient(u64), // fixme

    /// An request to switch to a workspace unknown to ToaruWM.
    #[error("Unknown workspace {0}")]
    UnknownWorkspace(String),

    /// An invalid point on the root window.
    #[error("Invalid point ({0}, {1})")]
    InvalidPoint(i32, i32),

    /// A name conflict in the given set of layouts.
    #[error("Namespace conflict: {0}")]
    NamespaceConflict(String),

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

impl From<io::Error> for ToaruError {
    fn from(e: io::Error) -> ToaruError {
        ToaruError::SpawnProc(e.to_string())
    }
}

/// The general result type used by ToaruWM.
pub type Result<T> = ::core::result::Result<T, ToaruError>;

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
    fn call(&self, state: ToaruState<'_, P, C>, err: ToaruError);
}
