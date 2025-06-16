//! Traits and structs for the platform the [WindowManager] runs on.
//! 
//! There are two possible platforms: X11 and Wayland. For more details
//! on how to use either platform, consult the module-level documentation.

use std::fmt::Debug;

use std::error::Error;

use crate::core::types::{
    ClientId, ClientData, Geometry
};
use crate::RuntimeConfig;
use crate::Toaru;

/// Backends for the X11 server.
pub mod x;
/// Backends for Wayland.
pub mod wayland;

pub mod output;

pub use output::Output;

/// An object that can serve as the backing platform of a [Toaru] instance.
/// 
/// There are three implementors of the `Platform` trait provided by this crate:
/// [`X11RBConn`], [`XCBConn`] (both X11-based), and [`Wayland`] (Wayland-based, duh).
/// 
/// # Handles
/// 
/// Since `Platform` is not dyn-compatible, a type implementing `Platform` must
/// support an associated type implementing `PlatformHandle`, which can be passed
/// around as a dyn object.
pub trait Platform: Debug {
    /// The client type that the platform uses to identify windows.
    type Client: ClientId + Debug;
    /// The error type returned by the platform.
    type Error: Error;
    /// The handle given out by the Platform.
    /// 
    /// The `Client` and `Error` associated types of the handle should match
    /// that of the parent platform.
    type Handle: PlatformHandle<Client = Self::Client, Error = Self::Error>;

    fn handle(&mut self) -> Self::Handle;

    /// The name of the platform. Returns the name of the platform's backend: for example,
    /// an X11 platform backed by xcb will return "xcb", while a Wayland platform
    /// running directly on a TTY will return "drm".
    /// 
    /// For checking whether the platform is X11 or Wayland, use `platform_type`.
    fn name(&self) -> &str;

    /// Returns the platform type, i.e. `PlatformType::Wayland` if running on Wayland,
    /// and `PlatformType::X11`` if running as an X11 window manager.
    fn platform_type(&self) -> PlatformType;

    /// Whether the platform is currently running the Toaru instance as a window nested
    /// inside another compositor/window manager.
    fn nested(&self) -> bool;

    /// Runs the event loop, responding to events as they come in.
    fn run_event_loop<P, C>(&mut self, toaru: &mut Toaru<P, C>) -> Result<(), Self::Error>
    where
        P: Platform,
        C: RuntimeConfig;

    
}

/// A dyn-compatible handle to a platform.
pub trait PlatformHandle: Debug {
    type Client: ClientId + Debug;

    type Error: Error;

    /// The name of the platform. Returns the name of the platform's backend: for example,
    /// an X11 platform backed by xcb will return "xcb", while a Wayland platform
    /// running directly on a TTY will return "drm".
    /// 
    /// For checking whether the platform is X11 or Wayland, use `platform_type`.
    fn name(&self) -> &str;

    /// Returns the platform type, i.e. `PlatformType::Wayland` if running on Wayland,
    /// and `PlatformType::X11`` if running as an X11 window manager.
    fn platform_type(&self) -> PlatformType;

    /// Whether the platform is currently running the Toaru instance as a window nested
    /// inside another compositor/window manager.
    fn nested(&self) -> bool;

    fn all_outputs(&self) -> Result<&[Output], Self::Error>;

    fn query_tree(&self, client: &Self::Client) -> Result<Geometry, Self::Error>;

    fn query_pointer(&self);

    fn query_client_data(&self, clid: &Self::Client) -> Result<ClientData, Self::Error>;
}

/// Convenience type alias for working with PlatformHandle dyn objects.
pub type PlatformHandleDyn<P: Platform> = dyn PlatformHandle<Client = P::Client, Error = P::Error>;

/// The type of platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformType {
    /// The platform is running on X11.
    X11,
    /// The platform is running on Wayland.
    Wayland,
}