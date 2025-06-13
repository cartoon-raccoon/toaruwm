//! Traits and structs for the backend of the [WindowManager].

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