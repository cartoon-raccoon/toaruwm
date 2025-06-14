use smithay::backend::{
    drm::DrmDeviceFd, egl::context::ContextPriority, renderer::{
        gles::GlesRenderer, 
        multigpu::{gbm::GbmGlesBackend, GpuManager, MultiRenderer}
    }
};

#[derive(Debug)]
pub struct DrmBackend {
    pub(crate) gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
}

pub type DrmRenderer<'render> = MultiRenderer<
    'render,
    'render,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>>;

impl DrmBackend {
    pub fn new() -> Self {
    
        let api = GbmGlesBackend::with_context_priority(ContextPriority::High);
        let gpu_manager = GpuManager::new(api).unwrap(); // fixme

        Self {
            gpu_manager
        }
    }
}