//! This module contains the core types used within toaruwm.
//! It contains high-level types that are used directly by toaruwu
//! to manage windows and clients.

/// Basic types used throughout toaruwm.
pub mod types;
/// A ringbuffer type used throughout toaruwm.
pub mod ring;
/// Types used for desktop management.
pub mod desktop;
/// Types used within workspaces.
pub mod workspace;
/// Types used to represent individual windows.
pub mod window;

pub use ring::{Ring, Selector};
pub use desktop::{Screen, Desktop};
pub use workspace::Workspace;
pub use window::{ClientRing, Client};