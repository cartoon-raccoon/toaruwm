//! Core data types used by ToaruWM.
//!
//! This module contains the core types used within toaruwm.
//! It contains high-level types that are used directly by toaruwu
//! to manage windows and clients.

/// Types used for desktop management.
pub mod desktop;
/// A ringbuffer type used throughout toaruwm.
pub mod ring;
/// Basic types used throughout toaruwm.
pub mod types;
/// Types used to represent individual windows.
pub mod window;
/// Types used within workspaces.
pub mod workspace;

#[doc(inline)]
pub use desktop::{Desktop, Screen};
#[doc(inline)]
pub use ring::{Ring, Selector};
#[doc(inline)]
pub use window::{Client, ClientRing};
#[doc(inline)]
pub use workspace::{Workspace, WorkspaceSpec};
