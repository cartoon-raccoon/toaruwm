use thiserror::Error;

use smithay::backend::{
    session::{
        Session,
        libseat::{
            LibSeatSession, LibSeatSessionNotifier, Error as LibSeatError
        }
    },
    udev::{self, UdevBackend},
    drm::{DrmNode, DrmDeviceFd, CreateDrmNodeError}, 
    egl::context::ContextPriority, renderer::{
        gles::GlesRenderer, 
        multigpu::{gbm::GbmGlesBackend, GpuManager, MultiRenderer}
    }
};

use crate::backend::wayland::WaylandError;
use super::WaylandBackend;

#[derive(Debug)]
pub struct DrmBackend {
    pub(crate) seat_name: String,
    pub(crate) libseat: LibSeatSession,
    pub(crate) libseat_notifier: LibSeatSessionNotifier,
    pub(crate) udev: UdevBackend,
    pub(crate) primary_node: DrmNode,
    pub(crate) gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
}

pub type DrmRenderer<'render> = MultiRenderer<
    'render,
    'render,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>>;

impl DrmBackend {
    pub fn new() -> Result<Self, WaylandError> {

        //todo: calloop integration, libinput

        // create our libseat session and acquire seat.
        let (libseat, notifier) = LibSeatSession::new()?;
        let seat_name = libseat.seat();

        // connect to udev
        let udev = UdevBackend::new(&seat_name)
            .map_err(|e| WaylandError::UdevErr(e.to_string()))?;

        // acquire the primary rendering node
        let primary_node = udev::primary_gpu(&seat_name)
            .map_err(|e| WaylandError::UdevErr(e.to_string()))?
            .ok_or(WaylandError::UdevErr(String::from("unable to determine primary gpu")))
            .and_then(|node_path| DrmNode::from_path(node_path)
                .map_err(|e| DrmError::CreateError(e).into())
            )?;

        // create the gpu manager for multiple gpu rendering
        let api = GbmGlesBackend::with_context_priority(ContextPriority::High);
        let gpu_manager = GpuManager::new(api).unwrap(); // fixme

        Ok(Self {
            seat_name,
            libseat,
            libseat_notifier: notifier,
            udev,
            primary_node,
            gpu_manager
        })
    }
}

impl WaylandBackend for DrmBackend {
    fn name(&self) -> &str {
        "drm"
    }

    fn render(&mut self) {
        
    }
}

impl From<LibSeatError> for WaylandError {
    fn from(e: LibSeatError) -> WaylandError {
        WaylandError::SessionErr(e)
    }
}

#[derive(Debug, Error)]
pub enum DrmError {
    #[error("could not create drm node: {0}")]
    CreateError(CreateDrmNodeError),
}


impl From<DrmError> for WaylandError {
    fn from(e: DrmError) -> WaylandError {
        WaylandError::DrmError(e)
    }
}