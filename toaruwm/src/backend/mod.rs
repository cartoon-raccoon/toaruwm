//! Traits and structs for the backend of the [WindowManager].

use thiserror::Error;

use self::x::XError;
use self::wayland::WaylandError;

/// Backends for the X11 server.
pub mod x;
/// Backends for Wayland.
pub mod wayland;

/// An object that can serve as the backend of a [WindowManager].
pub trait Backend {
    /// The name of the backend.
    fn name(&self) -> &str;

    /// Runs the event loop, responding to events as they come in.
    fn run_event_loop(&mut self);

    
}

/// An error raised by the backend. It encapsulates an error raised by
/// either type of backend.
#[derive(Debug, Error)]
pub enum BackendError {
    /// The error raised by an X11 backend.
    /// 
    /// This variant is only ever returned when an X11 backend is in use.
    #[error(transparent)]
    XError(XError),
    /// The error raised by a Wayland backend.
    /// 
    /// This variant is only every returned when a Wayland backend is in use.
    #[error(transparent)]
    WaylandError(WaylandError),
}

impl BackendError {
    /// Returns an immutable reference to the contained XError type, if present.
    /// 
    /// Returns `Some` if it is an `XError``, `None` otherwise.
    pub fn as_ref_xerror(&self) -> Option<&XError> {
        match self {
            Self::XError(e) => Some(e),
            _ => None
        }
    }

    /// Returns a mutable reference to the contained XError type, if present.
    /// 
    /// Returns `Some` if it is an `XError``, `None` otherwise.
    pub fn as_ref_xerror_mut(&mut self) -> Option<&mut XError> {
        match self {
            Self::XError(e) => Some(e),
            _ => None
        }
    }

    /// Returns an immutable reference to the contained WaylandError type, if present.
    /// 
    /// Returns `Some` if it is a `WaylandError`, `None` otherwise.
    pub fn as_ref_wlerror(&self) -> Option<&WaylandError> {
        match self {
            Self::WaylandError(e) => Some(&e),
            _ => None
        }
    }

    /// Returns a mutable reference to the contained WaylandError type, if present.
    /// 
    /// Returns `Some` if it is a `WaylandError`, `None` otherwise.
    pub fn as_ref_wlerror_mut(&mut self) -> Option<&mut WaylandError> {
        match self {
            Self::WaylandError(e) => Some(e),
            _ => None
        }
    }
}