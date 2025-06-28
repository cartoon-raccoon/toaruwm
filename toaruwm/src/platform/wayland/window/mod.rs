//! Functionality for managing Wayland windows. 

mod mapped;
mod unmapped;

#[doc(inline)]
pub use mapped::*;

#[doc(inline)]
pub use unmapped::*;

use smithay::reexports::wayland_server::{
    protocol::wl_surface::WlSurface
};
use smithay::backend::renderer::{
    utils::with_renderer_surface_state,
};

/// Checks whether a given `WlSurface` is mapped (i.e. it has sent a `wl_surface.commit`)
/// request and the compositor has done so.
pub fn is_mapped(surface: &WlSurface) -> bool {
    // if the surface hasn't committed yet, the call to buffer will be None.
    with_renderer_surface_state(surface, |state| state.buffer().is_some()).unwrap_or(false)
}