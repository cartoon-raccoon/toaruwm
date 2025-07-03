//! Core data types used by ToaruWM.
//!
//! This module contains the core types used within toaruwm.
//! It contains high-level types that are used directly by toaruwu
//! to manage windows and clients.

pub mod desktop;
pub mod ring;
pub mod types;
pub mod window;
pub mod workspace;

#[doc(inline)]
pub use desktop::{Desktop, Monitor, WorkspaceMux, WorkspaceMuxHandle};
#[doc(inline)]
pub use ring::{Ring, Selector};
#[doc(inline)]
pub use window::{Window, WindowRing};
#[doc(inline)]
pub use workspace::{Workspace, WorkspaceSpec};
