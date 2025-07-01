//! Types and helpers for configuring `Toaru`.

pub mod output;
pub mod rules;
pub mod runtime;
pub mod section;

mod config;

pub use output::{Output, OutputLayout, OutputMode, OutputScale};
pub use runtime::{RuntimeConfig};
pub use section::ConfigSection;

#[doc(inline)]
pub use config::*;

use crate::core::WorkspaceSpec;
use crate::layouts::Layout;

/// A trait defining a `WindowManager` configuration.
///
/// On initialization, the `WindowManager` queries a config
/// for various fields to move elsewhere, before at the end converting
/// the config into a runtime configuration.
///
/// You will probably have noticed that the `workspaces` and
/// `layouts` methods take a `&mut self`, but return owned
/// types. This is because any type implementing this trait
/// is expected to be dropped by the end of the window manager
/// initialization process, where it will be turned into a
/// type implementing `RuntimeConfig`. Thus, it can afford
/// to do some expensive operations such as cloning. These methods
/// also use a `&mut self` to allow the user to mutate internal
/// state, or to do something like [`std::mem::take`].
///
/// The type implementing this trait must yield these key fields
/// for the window manager to take and initialize
/// itself, before converting itself into a `RuntimeConfig`.
///
/// # Configuration Keys
///
/// The required configuration keys are:
///
/// - *Layouts*: the set of layouts used by the window manager.
/// - *Workspaces*: the workspaces to be created by the window
/// manager.
///
/// The following are not explicitly required by `Config`, but
/// are required by the [`RuntimeConfig`] trait, which `Self::Config`
/// needs to implement:
///
/// - *Float Classes*: the set of window classes that the window
/// manager will not place under layout.
/// - *Border Pixel*: The thickness of the window border.
/// - *Window Gaps*: The gaps between windows.
/// - *Focus Follows Pointer* Whether the focused window should follow the
/// pointer's position on the screen.
/// - *Unfocused*: The border color of unfocused windows.
/// - *Focused*: The border color of focused windows.
/// - *Urgent*: The border color of focused windows.
///
/// `RuntimeConfig` also requires a method `get_key` to retrieve arbitrary
/// values of keys.
///
/// # Validity
///
/// While user-defined keys may have their own invariants that
/// should not violated, A type `Config` also has one invariant of its own,
/// that its Layouts and Workspaces must contain at least one member,
/// i.e. they cannot be empty.
pub trait Config<P> {
    /// The type it will finally convert itself into.
    type Runtime: RuntimeConfig;

    /// The workspace collection returned when queried.
    type Workspaces: IntoIterator<Item = WorkspaceSpec>;

    /// The layout collection returned when queried.
    type Layouts: IntoIterator<Item = Box<dyn Layout<P>>>;

    /// Yield an iterator over the workspaces.
    fn take_workspaces(&mut self) -> Self::Workspaces;

    /// Yield an iterator over the layouts.
    fn take_layouts(&mut self) -> Self::Layouts;

    /// Perform the conversion into the RuntimeConfig.
    fn into_runtime_config(self) -> Self::Runtime;
}