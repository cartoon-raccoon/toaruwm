//! Traits and structs for the platform the [WindowManager] runs on.
//! 
//! There are two possible platforms: X11 and Wayland. For more details
//! on how to use either platform, consult the module-level documentation.

use std::fmt::Debug;

use std::error::Error;

use crate::core::types::{
    ClientId, ClientData, Rectangle, Logical
};

/// Backends for the X11 server.
pub mod x11;
/// Backends for Wayland.
pub mod wayland;
/// Output representation.
pub mod output;

pub use output::Output;

/// An object that can serve as the backing platform of a [Toaru] instance.
/// 
/// There are three implementors of the `Platform` trait provided by this crate:
/// [`X11RBConn`], [`XCBConn`] (both X11-based), and [`Wayland`] (Wayland-based, duh).
pub trait Platform: Debug {
    /// The client type that the platform uses to identify windows.
    type Client: ClientId + Debug;
    /// The error type returned by the platform.
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
    /// 
    /// This should only ever return true if the platform is Wayland. When running as a nested
    /// window manager in Xephyr, there is generally no mechanism to check if the instance
    /// is nested or not.
    fn nested(&self) -> bool;

    /// Return a view into all available outputs that the platform can see.
    fn all_outputs(&self) -> Result<&[Output], Self::Error>;

    /// Query the client tree.
    fn query_tree(&self, client: &Self::Client) -> Result<Rectangle<Logical>, Self::Error>;

    /// Query the pointer.
    fn query_pointer(&self);

    /// Query the client data.
    fn query_client_data(&self, clid: &Self::Client) -> Result<ClientData, Self::Error>;
}

/// The type of platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformType {
    /// The platform is running on X11.
    X11,
    /// The platform is running on Wayland.
    Wayland,
}