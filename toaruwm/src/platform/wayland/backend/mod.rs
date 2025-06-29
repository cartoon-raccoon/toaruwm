use std::fmt::Debug;
use std::error::Error;

use smithay::reexports::{
    wayland_server::{
        protocol::wl_surface::WlSurface,
        DisplayHandle
    },
};
use smithay::backend::allocator::dmabuf::Dmabuf;

pub mod drm;
pub mod winit;

pub use drm::{DrmBackend, DrmBackendError};
pub use winit::WinitBackend;

use super::util::IdCounter;
use super::{WaylandImpl, WaylandError};

use crate::manager::state::{RuntimeConfig};
use crate::types::Dict;

static OUTPUT_ID_COUNTER: IdCounter = IdCounter::new();

/// A unique ID a
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct OutputId(u64);

impl OutputId {
    pub fn new() -> Self {
        Self(OUTPUT_ID_COUNTER.next())
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputName {
    pub connector: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
}

/// Returns whether a windowed backend or a TTY-based backend should be used,
/// depending on the state of the system.
pub fn should_run_nested() -> bool {
    true
}

/// A backend that manages input/output devices, rendering, and DRM access.
/// 
/// It exposes input and output devices, and sends events to handlers.
/// 
/// There are two implementors of `WaylandBackend` provided by this crate:
/// [`WinitBackend`] and [`DrmBackend`].
pub trait WaylandBackend: Debug {
    /// The name of the backend.
    fn name(&self) -> &str;

    /// The seat name used by the backend.
    fn seat_name(&self) -> &str;

    /// Render a frame and submit it for viewing.
    fn render(&mut self);

    /// Import a DMA-BUF handle into the renderer.
    /// 
    /// Return `true` on success, `false` otherwise.
    fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool;

    /// Function for optimizing buffer imports across multiple GPUs.
    /// 
    /// If you are working with a multiple-GPU topography (i.e. [`MultiRenderer`][1])
    /// you should reimplement this method.
    /// 
    /// See [`GpuManager::early_import`][2] in Smithay for details.
    /// 
    /// [1]: smithay::backend::renderer::multigpu::MultiRenderer
    /// [2]: smithay::backend::renderer::multigpu::GpuManager::early_import
    #[allow(unused_variables)]
    fn early_import(&mut self, surface: &WlSurface) {}
}

/// Initialize state that needs access to the internal fields of the `Wayland` struct.
/// 
/// Since all `WaylandBackend`s are created before the actual Wayland struct (`Wayland`
/// requires a `backend` parameter to be created), there might be some state that you need
/// to initialize that cannot be done without access to the objects owned by the `Wayland`
/// struct, such as the display handle or compositor state. If that is the case, re-implement
/// this method to do so.
/// 
/// This method is called when you call `Wayland::new()`, to initialize any state in your
/// backend that requires access to a `Wayland` instance.
pub trait WaylandBackendInit<C: RuntimeConfig>: WaylandBackend + Debug {
    /// Initialize Wayland State
    #[allow(unused_variables)]
    fn init(
        &mut self,
        display: DisplayHandle,
        wl_impl: &mut WaylandImpl<C, Self>,
        args: Dict)-> Result<(), WaylandError>
    where
        Self: Sized,
        C: RuntimeConfig,
    { Ok(()) }
}

/// An error type provided by a wayland backend.
pub trait WaylandBackendError: Error {
    /// The name of the backend that provided this error
    fn backend_name(&self) -> &str;
}
