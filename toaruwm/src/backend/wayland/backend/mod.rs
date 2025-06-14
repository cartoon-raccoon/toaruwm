pub mod drm;
pub mod winit;

pub use drm::DrmBackend;
pub use winit::WinitBackend;

/// The backend that manages input/output devices, rendering, and DRM access.
#[derive(Debug)]
pub enum Backend {
    /// The backend used when running from a TTY, using DRM.
    /// 
    /// This should be the most commonly used backend.
    Drm(DrmBackend),
    /// The backend used when running within a window, nested within another
    /// Wayland compositor or on the X11 server.
    Window(WinitBackend),
}

impl Backend {
    pub fn autocreate() -> Self {
        todo!()
    }

    pub fn new_drm() -> Self {
        todo!()
    }

    pub fn new_window() -> Self {
        todo!()
    }
}