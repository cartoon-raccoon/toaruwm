pub mod drm;
pub mod winit;

pub use drm::DrmBackend;
pub use winit::WinitBackend;

/// Automatically creates a new backend based on 
// pub fn backend_autocreate() -> impl WaylandBackend {
//     todo!()
// }

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
