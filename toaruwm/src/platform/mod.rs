//! Traits and structs for the backing platform of a [`Manager`][1] instance.
//! 
//! There are two possible platforms: [X11][2] and [Wayland][3]. For more details
//! on how to use either platform, consult the module-level documentation.
//! 
//! This module provides the base traits that a platform should implement, as well
//! as implementations of each platform.
//! 
//! ## The `Platform` trait
//! 
//! The core item in this module is the [`Platform`] trait. This defines a platform that
//! defines windows, manages outputs, and can implement the window management functionality
//! provided by a `Manager`.
//! 
//! ### Associated Types
//! 
//! Each `Platform` implementation has associated types that must implement certain other
//! traits that are also defined in this module. These include:
//! 
//! - `WindowId`: a [`Copy`]-able identifier that can uniquely identify a window. It must
//! implement the [`PlatformWindowId`] trait.
//! - `Window`: a type that represents a top-level window. It must implement the [`PlatformWindow`]
//! trait.
//! - `Output`: a type that represents a physical monitor connected to the machine. It must implement
//! the [`PlatformOutput`] trait.
//! 
//! For more details on these traits, consult their documentation.
//! 
//! [1]: crate::manager::Manager
//! [2]: crate::platform::x11
//! [3]: crate::platform::wayland
//! [4]: std::sync::Arc
//! [5]: smithay::desktop::Window
//! [6]: smithay::output::Output

use std::any::Any;
use std::hash::Hash;
use std::fmt::Debug;
use std::error::Error;

use crate::core::types::{
    ClientData, Rectangle, Point, Logical, Transform, Size
};
use crate::config::output::{OutputMode, OutputScale, OutputInfo};

/// Backends for the X11 server.
pub mod x11;
/// Backends for Wayland.
pub mod wayland;

use crate::types::Scale;

#[doc(inline)]
pub use wayland::Wayland;

/// A type that can uniquely identify a window connected to a [`Platform`].
/// 
/// It is associated with a type that implements [`PlatformWindow`].
/// 
/// A [`PlatformWindowId`] is meant to be easily passed around as a token that
/// can be later used to retrieve its corresponding `PlatformWindow`, and so
/// must implement [`Copy`] so that it can be efficiently passed around without
/// potentially expensive cloning operations.
pub trait PlatformWindowId: Debug + Copy + PartialEq + Eq + Hash {}

/// A type that represents a top-level window.
/// 
/// A `PlatformWindow` represents a window, as managed by the Platform it is
/// associated with. Each window should have an associated window ID that can be
/// used to easily identify it.
/// 
/// ## Window Configuration, Mapping, and Visibility
/// 
/// A `PlatformWindow` implementation must track three different states:
/// configuration, visibility, and map state. While these three states may appear
/// similar, they have some subtle differences.
/// 
/// ### Mapped and Unmapped windows
/// 
/// A mapped window is one that has been mapped onto the global coordinate space
/// as tracked internally by its associated [`Platform`]. It is always configured,
/// but may or may not be visible (e.g. it may be offscreen).
/// 
/// ### Configuration
/// 
/// A configured window is one that has been sent its initial geometry. It may be
/// unmapped.
/// 
/// ### Visibility
/// 
/// A visible window is one that is currently being displayed onscreen. It is always
/// mapped and configured, but a mapped and configured window may not be visible
/// (e.g. it may be offscreen).
pub trait PlatformWindow: Debug + Clone {
    /// The identifier used to refer to the window.
    type Id: PlatformWindowId;

    /// Returns the window's associated ID.
    fn id(&self) -> Self::Id;

    /// Whether the window has been configured.
    fn configured(&self) -> bool;

    /// Return the window's underlying geometry, if configured.
    /// 
    /// This should return the window's _committed_ geometry,
    /// not the pending geometry.
    fn geom(&self) -> Option<Rectangle<i32, Logical>>;

    /// Set the window's pending configuration.
    fn configure(&mut self, 
        pos: Option<Point<i32, Logical>>, 
        size: Option<Size<i32, Logical>>,
    );
}

/// A type that abstracts over an output, as used by a [`Platform`].
/// 
/// A `PlatformOutput` represents a physical monitor connected to the machine.
/// It provides information such as its location in the global coordinate space,
/// its name, etc.
pub trait PlatformOutput: Debug + Clone + Eq + Hash {
    /// The name of the Output.
    fn name(&self) -> String;

    /// Output information.
    fn info(&self) -> OutputInfo;

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

/// A platform Error type.
/// 
/// The type needs to implement [`Any`] so receivers of the error can attempt to downcast
/// it to its concrete type, if they need more info.
pub trait PlatformError: Error + Any {}

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
    type WindowId: PlatformWindowId;
    /// The type that the platform uses to represent top-level windows.
    type Window: PlatformWindow<Id = Self::WindowId>;
    /// The type used to represent an output (i.e. a physical monitor).
    type Output: PlatformOutput;
    /// The error type returned by the platform.
    type Error: PlatformError;

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
}



/// The type of platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlatformType {
    /// The platform is running on X11.
    X11,
    /// The platform is running on Wayland.
    Wayland,
}