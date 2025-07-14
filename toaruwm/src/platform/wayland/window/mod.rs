//! Functionality for managing Wayland windows. 

mod unmapped;

#[doc(inline)]
pub use unmapped::*;

use smithay::reexports::wayland_server::{
    protocol::wl_surface::WlSurface
};
use smithay::backend::renderer::{
    utils::with_renderer_surface_state,
};
use smithay::desktop::Window;

use crate::platform::{PlatformWindow, wayland::WaylandWindowId};
use crate::types::{Rectangle, Point, Size, Logical};


use super::util::IdCounter;

/// Checks whether a given `WlSurface` is mapped (i.e. it has sent a `wl_surface.commit`)
/// request and the compositor has done so.
pub fn is_mapped(surface: &WlSurface) -> bool {
    // if the surface hasn't committed yet, the call to buffer will be None.
    with_renderer_surface_state(surface, |state| state.buffer().is_some()).unwrap_or(false)
}

pub(crate) static WINDOW_ID_GENERATOR: IdCounter = IdCounter::new();

/// A window as represented by the Wayland platform.
/// 
/// This type is created internally by the `Wayland` platform.
#[derive(Debug, Clone)]
pub struct WaylandWindow {
    // We can safely use this struct without wrapping it in an Arc, since
    // identifier is Copy and cannot be modified at any point
    identifier: WaylandWindowId,
    inner: Window,
}

impl PlatformWindow for WaylandWindow {
    type Id = WaylandWindowId;

    fn id(&self) -> Self::Id {
        self.identifier
    }

    fn configured(&self) -> bool {
        self.inner.toplevel().map(|tl| tl.is_initial_configure_sent()).is_some_and(|b| b)
    }

    fn geom(&self) -> Option<Rectangle<i32, Logical>> {
        if self.configured() {
            Some(self.inner.geometry().into())
        } else {
            None
        }
    }

    fn configure(&mut self, 
        _pos: Option<Point<i32, Logical>>, 
        size: Option<Size<i32, Logical>>
    ) {
        // ignore pos, get it from the Manager
        self.inner.toplevel().map(|tl| tl.with_pending_state(|tlstate| {
            if let Some(size) = size {
                tlstate.size = Some((size.width, size.height).into())
            }
        }));
    }
}

impl WaylandWindow {
    pub(crate) fn new(id: WaylandWindowId, window: Window) -> Self {
        Self {
            identifier: id,
            inner: window,
        }
    }
}
