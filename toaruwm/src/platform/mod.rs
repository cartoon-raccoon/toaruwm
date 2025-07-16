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
/// Backends for the X11 server.
pub mod x11;