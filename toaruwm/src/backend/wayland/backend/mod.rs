pub mod drm;
pub mod winit;

pub use drm::{DrmBackend, DrmError};
pub use winit::WinitBackend;

use super::WaylandError;

/// Automatically creates a new backend based on 
pub(crate) fn backend_autocreate() -> Result<impl WaylandBackend, WaylandError> {
    DrmBackend::new()
}

/// A backend that manages input/output devices, rendering, and DRM access.
/// 
/// It exposes input and output devices, and sends events to handlers.
/// 
/// There are two implementors of `WaylandBackend`: [`WinitBackend`] and
/// [`DrmBackend`].
pub trait WaylandBackend {
    /// The name of the backend.
    fn name(&self) -> &str;

    /// Render a frame and submit it for viewing.
    fn render(&mut self);
}
