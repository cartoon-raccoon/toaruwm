//! Traits and structs for the backing platform of a [`Toaru`][1] instance.
//! 
//! There are two possible platforms: X11 and Wayland. For more details
//! on how to use either platform, consult the module-level documentation.
//! 
//! [1]: crate::Toaru

use std::hash::Hash;
use std::fmt::Debug;
use std::io::Error as IoError;
use std::error::Error;

use crate::core::types::{
    ClientData, Rectangle, Point, Logical, Transform, Size
};
use crate::config::output::{OutputMode, OutputScale};

/// Backends for the X11 server.
pub mod x11;
/// Backends for Wayland.
pub mod wayland;

use crate::types::Scale;
use crate::util::spawn;

#[doc(inline)]
pub use wayland::Wayland;

/// A type that can uniquely identify any window connected to a
/// running Toaru instance.
/// 
/// It is backend-agnostic, and each backend provides their own
/// type that implements this trait.
pub trait PlatformWindowId: Debug + Copy + Eq + Hash {}

/// A type that abstracts over an output, as used by a Platform.
pub trait PlatformOutput {
    /// The name of the Output.
    fn name(&self) -> String;

    /// The location of the Output in the 2D coordinate space.
    fn location(&self) -> Point<i32, Logical>;

    /// The transformation currently applied to this Output.
    fn transform(&self) -> Transform;
    
    /// The scale currently used by the Output.
    fn scale(&self) -> OutputScale;

    /// The current mode of the output (resolution and refresh rate in Hz).
    fn current_mode(&self) -> Option<OutputMode>;

    /// The preferred mode of the output.
    fn preferred_mode(&self) -> Option<OutputMode>;

    /// A Vec over the Output's available modes.
    fn modes(&self) -> Vec<OutputMode>;

    /// Returns the Geometry of the Output, if its mode is set.
    fn geometry(&self) -> Option<Rectangle<i32, Logical>> {
        let Some(mode) = self.current_mode() else {
            return None
        };

        let size = match self.scale() {
            OutputScale::Fractional(f) | OutputScale::Split {fractional: f, ..} => {
                let Size {width, height, .. } = mode.size.as_f64().as_logical(Scale::uniform(f));
                Size::<i32, Logical>::new(width as i32, height as i32)
            }
            OutputScale::Integer(i) => {
                mode.size.as_logical(Scale::uniform(i))
            }
        };

        Some(Rectangle {point: self.location(), size})
    }
}

/// An object that can serve as the backing platform of a [`Toaru`][1] instance.
/// 
/// A `Platform` implementation usually provides the link to the underlying graphics
/// stack, and works to actually implement the functionality of its `Toaru` instance.
/// For example, an X11-based platform contains an implementation of a connection to the
/// X Server, and handles sending requests, and events from the server. A Wayland-based
/// platform implements the Wayland protocol and associated extension protocols, and
/// provides compositing functionality with the DRM subsystem, or running as a nested server.
/// 
/// There are three implementors of the `Platform` trait provided by this crate:
/// [`X11RBConn`], [`XCBConn`] (both X11-based), and [`Wayland`] (Wayland-based, duh).
/// 
/// [1]: crate::Toaru
pub trait Platform: Debug {
    /// The client type that the platform uses to identify windows.
    type WindowId: PlatformWindowId + Debug;
    /// The type used to represent an output (i.e. a physical monitor).
    type Output: PlatformOutput + Debug;
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
    fn all_outputs(&self) -> Result<&[Self::Output], Self::Error>;

    /// Query the client tree.
    fn query_tree(&self, client: Self::WindowId) -> Result<Rectangle<i32, Logical>, Self::Error>;

    /// Query the pointer.
    fn query_pointer(&self);

    /// Query the client data.
    fn query_window_data(&self, clid: Self::WindowId) -> Result<ClientData, Self::Error>;

    /// Spawn an external command that runs independently of the Platform instance.
    fn spawn_external(&self, command: &str, args: &[&str]) -> Result<(), IoError> {
        Ok(())
    }
}



/// The type of platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformType {
    /// The platform is running on X11.
    X11,
    /// The platform is running on Wayland.
    Wayland,
}