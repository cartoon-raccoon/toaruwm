//! A DRM backend for [`Wayland`].
//! 
//! This module provides [`DrmBackend`], a backend which is meant to run in a TTY.

use std::path::Path;
use std::collections::{HashMap, HashSet};

use thiserror::Error;
use tracing::{debug, warn, info, error};

use smithay::reexports::{
    calloop::{Dispatcher, LoopHandle, RegistrationToken}, 
    drm::{
        control::{crtc, connector}, 
        node::NodeType
    },
    input::Libinput, rustix::fs::OFlags,
    wayland_server::{
        DisplayHandle,
        protocol::wl_surface::WlSurface,
    },
};
use smithay::backend::{
    allocator::{
        Fourcc,
        gbm::{GbmAllocator, GbmDevice, GbmBufferFlags},
        dmabuf::Dmabuf,
    },
    egl::{EGLDevice, EGLDisplay, Error as SmithayEglError},
    session::{
        Session,
        libseat::{
            LibSeatSession, Error as LibSeatSessionError,
        },
        Event as SessionEvent
    },
    udev::{self, UdevBackend, UdevEvent},
    drm::{
        compositor::DrmCompositor,
        output::DrmOutputManager,
        DrmNode, DrmDevice, DrmDeviceFd,
        CreateDrmNodeError, DrmError as SmithayDrmError,
        DrmEvent, DrmEventMetadata,
    },
    libinput::{LibinputSessionInterface, LibinputInputBackend},
    input::{InputEvent},
    egl::context::ContextPriority,
    renderer::{
        gles::GlesRenderer, 
        multigpu::{gbm::GbmGlesBackend, GpuManager, MultiRenderer},
        ImportMemWl, ImportEgl, ImportDma,
    }
};
use smithay::desktop::{
    utils::OutputPresentationFeedback
};
use smithay::output::Output;
use smithay::utils::{
    DeviceFd,
};
use smithay_drm_extras::drm_scanner::{DrmScanner, DrmScanEvent};

use crate::platform::wayland::{
    WaylandError, Wayland, WaylandImpl,
    backend::WaylandBackendError
};
use crate::platform::wayland::prelude::*;
use crate::types::Dict;
use super::{WaylandBackend, WaylandBackendInit, OutputId, OutputName, super::state::WlState};

/*
THINGS THAT A DRM-BACKED WAYLAND COMPOSITOR NEEDS TO INITIALIZE
objects in play:
a. Wayland
    1. WlState
    2. WaylandBackend

1. Create libseat session and notifier (backend)

2. Acquire primary GPU (backend)

3. Create the GPU manager (backend)

4. Create udev backend (backend)

5. Create libinput backend (backend)

6. Bind libseat notifier and libinput to event loop (backend)

7. Create the primary node (rendering? display?)

8. Add primary node to state tracking (Wayland or WaylandBackend)

9. Add every node to state tracking

10. Update SHM state (owned by WlState) with SHM formats (provided by WaylandBackend)

11. Create renderer from GPUs and bind to wl_display

12. Init DMA-BUF support with format list from primary gpu (through renderer)

13. 


Standard


*/

const SUPPORTED_FORMATS: &[Fourcc] = &[
    Fourcc::Abgr2101010,
    Fourcc::Argb2101010,
    Fourcc::Abgr8888,
    Fourcc::Argb8888,
    Fourcc::Xbgr2101010,
    Fourcc::Xrgb2101010,
];

#[derive(Debug)]
pub struct DrmBackend<C: RuntimeConfig + 'static> {
    /// The seat name.
    pub(crate) seat_name: String,
    /// A handle to the underlying session.
    pub(crate) session: LibSeatSession,
    /// A libinput context.
    pub(crate) libinput: Libinput,
    /// Udev dispatcher.
    pub(crate) udev_dispatcher: Dispatcher<'static, UdevBackend, Wayland<C, DrmBackend<C>>>,
    /// The primary node on which all is displayed.
    pub(crate) primary_node: DrmNode,
    /// The primary node on which stuff is rendered.
    pub(crate) primary_render: DrmNode,
    pub(crate) gpu_manager: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    pub(crate) devices: HashMap<DrmNode, DrmOutputDevice>,
    pub(crate) loophandle: LoopHandle<'static, Wayland<C, DrmBackend<C>>>
}

pub type DrmRenderer<'render> = MultiRenderer<
    'render,
    'render,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>>;

pub(crate) type GbmDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmDevice<DrmDeviceFd>,
    Option<OutputPresentationFeedback>,
    DrmDeviceFd
>;

impl<C: RuntimeConfig> DrmBackend<C> {
    /// Creates a new DrmBackend.
    pub fn new(handle: LoopHandle<'static, Wayland<C, Self>>) -> Result<Self, WaylandError> {

        // create our libseat session and acquire seat.
        let (libseat, seatnotifier) = LibSeatSession::new()?;
        let seat_name = libseat.seat();
        info!("Creating session on seat {seat_name}");

        // connect to udev.
        let udev_backend = UdevBackend::new(&seat_name)
            .map_err(|e| WaylandError::UdevErr(e.to_string()))?;

        let udev_dispatcher = Dispatcher::new(udev_backend, move |event, _, wayland: &mut Wayland<C, DrmBackend<C>>| {
            wayland.backend.on_udev_event(event, &mut wayland.wl_impl);
        });
        handle.register_dispatcher(udev_dispatcher.clone()).expect("could not register udev dispatcher");

        // acquire the primary gpu
        let primary_gpu = udev::primary_gpu(&seat_name) // fixme: return error instead of panicking
            .map_err(|e| WaylandError::UdevErr(e.to_string()))?
            // get the path of the primary gpu from udev, and try to convert it into a DrmNode
            .and_then(|path| DrmNode::from_path(path).ok()?.node_with_type(NodeType::Primary)?.ok())
            // if that fails, search all gpus to find one that works
            .unwrap_or_else(|| {
                udev::all_gpus(&seat_name)
                    .expect("could not get GPUs")
                    .into_iter()
                    .find_map(|x| DrmNode::from_path(x).ok())
                    .expect("no GPUs found on system")
            });

        info!("using {primary_gpu} as primary gpu");


        // create the gpu manager for multiple gpu rendering
        let api = GbmGlesBackend::with_context_priority(ContextPriority::High);
        let gpu_manager = GpuManager::new(api).expect("could not create GPU manager"); // fixme: handle error

        // create libinput backend

        let mut libinputctxt = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(
            libseat.clone().into()
        );

        let inputbackend = LibinputInputBackend::new(libinputctxt.clone());

        libinputctxt.udev_assign_seat(&seat_name)
            .map_err(|_| 
                DrmBackendError::InputError(format!("unable to assign seat {} to libinput context", 
                seat_name)))?;

        /* Bind event loop sources */

        // bind libinput backend
        handle.insert_source(inputbackend, |event, _, wayland| {
            wayland.handle_input_event(event);
        }).expect("unable to bind libinput backend");

        // bind libseat notifier
        handle.insert_source(seatnotifier, |event, _, wayland| {
            wayland.backend.on_session_event(event);
        }).expect("unable to bind libseat notifier");

        // todo: get render node from config or env var
        let primary_render = primary_gpu.node_with_type(NodeType::Render)
            .ok_or(DrmBackendError::DrmDeviceError(format!("could not create render node from primary GPU")))?
            .map_err(|e| DrmBackendError::from(e))?;

        Ok(Self {
            seat_name,
            session: libseat,
            libinput: libinputctxt,
            udev_dispatcher,
            primary_node: primary_gpu,
            primary_render,
            gpu_manager,
            devices: HashMap::new(),
            loophandle: handle
        })
    }

    fn device_added(&mut self, node: DrmNode, path: &Path, wl: &mut WaylandImpl<C, Self>) -> Result<(), DrmBackendError> {
        // open a new DRM device with our session handle
        let flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags:: NONBLOCK;
        let fd = self.session.open(path, flags)?;
        let drmdevfd = DrmDeviceFd::new(DeviceFd::from(fd));

        // create a new DRM device and link it to GBM
        let (drm, drm_notifier) = DrmDevice::new(drmdevfd.clone(), true)?;
        let gbm = GbmDevice::new(drmdevfd).map_err(|e| DrmBackendError::GbmError(e))?;

        // create a new EGL display from our GBM device
        // SAFETY: no other code besides smithay is using EGL, so we won't have our context
        // rugpulled out from under us.
        let egldisplay = unsafe { EGLDisplay::new(gbm.clone())? };
        let egldev = EGLDevice::device_for_display(&egldisplay)?;

        let render_node = egldev.try_get_render_node()?;

        let allocator = if let Some(rnode) = render_node {
            self.gpu_manager
                .as_mut()
                .add_node(rnode, gbm.clone())?;

            GbmAllocator::new(gbm.clone(), GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT)
        } else {
            self.devices
                .get(&self.primary_node)
                .or_else(|| {
                    self.devices
                        .values()
                        .find(|dev| dev.render_node == self.primary_node)
                })
                .map(|dev| GbmAllocator::new(gbm.clone(), GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT)) // fixme
                .ok_or(DrmBackendError::DrmDeviceError("could not find primary GPU".into()))?
        };

        self.loophandle.insert_source(drm_notifier, move |event, meta, wayland| {
            match event {
                DrmEvent::VBlank(crtc) => {
                    let meta = meta.expect("VBlank events must have metadata");
                    wayland.backend.on_vblank(crtc, meta);
                }
                DrmEvent::Error(e) => {
                    warn!("DRM error: {e}");
                }
            }
        }).expect("could not add vblank handler");

        //todo: insert new OutputDevice into devices

        self.device_changed(node);

        Ok(())
    }

        fn device_changed(&mut self, node: DrmNode) {
        self.connector_connected();

    }

    fn device_removed(&mut self, node: DrmNode) {
        self.connector_disconnected();
    }

    fn connector_connected(&mut self) {

    }

    fn connector_disconnected(&mut self) {

    }

    fn render(&mut self, node: DrmNode, crtc: Option<crtc::Handle>) {
        
    }
    
    fn on_udev_event(&mut self, event: UdevEvent, wl: &mut WaylandImpl<C, Self>) {
        match event {
            UdevEvent::Added { device_id, path } => {
                match DrmNode::from_dev_id(device_id) {
                    Ok(node) => {
                        // fixme: awful error handling
                        self.device_added(node, &path, wl).unwrap_or_else(|e| {
                            error!("error while adding device {path:?}: {e}");
                            ()
                        })
                    },
                    Err(e) => {

                    }
                }
            }
            UdevEvent::Changed { device_id } => {
                match DrmNode::from_dev_id(device_id) {
                    Ok(node) => self.device_changed(node),
                    Err(e) => {

                    }
                }
                //wayland.device_changed(node);
            }
            UdevEvent::Removed { device_id } => {
                match DrmNode::from_dev_id(device_id) {
                    Ok(node) => self.device_removed(node),
                    Err(e) => {}
                }
            }
        }
    }

    fn on_session_event(&mut self, event: SessionEvent) {
        match event {
            SessionEvent::PauseSession => {

            }
            SessionEvent::ActivateSession => {

            }
        }
    }

    fn on_vblank(&mut self, _crtc: crtc::Handle, _metadata: DrmEventMetadata) {

    }
}

impl<C: RuntimeConfig> WaylandBackend for DrmBackend<C> {
    fn name(&self) -> &str {
        "drm"
    }

    fn seat_name(&self) -> &str {
        &self.seat_name
    }

    fn render(&mut self) {
        
    }

    fn early_import(&mut self, surface: &WlSurface) {
        if let Err(e) = self.gpu_manager.early_import(self.primary_render, surface) {
            warn!("error doing early import: {e}");
        }
    }

    fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        let mut renderer = match self.gpu_manager.single_renderer(&self.primary_render) {
            Ok(r) => r,
            Err(e) => {
                debug!("error creating renderer for the primary GPU: {e:?}");
                return false;
            }
        };

        match renderer.import_dmabuf(dmabuf, None) {
            Ok(_tex) => {
                dmabuf.set_node(Some(self.primary_render));
                true
            }
            Err(e) => {
                warn!("error importing DMA-BUF: {e}");
                false
            }
        }
    }
}

impl<C: RuntimeConfig> WaylandBackendInit<C> for DrmBackend<C> {
    fn init(
        &mut self,
        display: DisplayHandle,
        wl_impl: &mut WaylandImpl<C, Self>,
        _args: Dict)-> Result<(), WaylandError>
    where
        Self: Sized,
    { 
        let mut renderer = self.gpu_manager
            .single_renderer(&self.primary_node)
            .unwrap();

        wl_impl.state.shm_state.update_formats(
            renderer.shm_formats()
        );

        renderer.bind_wl_display(&display).expect("unable to bind display to EGL renderer");

        for (device_id, path) in self.udev_dispatcher.clone().as_source_ref().device_list() {
            let node = DrmNode::from_dev_id(device_id)
                .expect("could not create DRM node");
            if let Err(e) = self.device_added(node, path, wl_impl) {
                warn!("error while adding device: {e:?}");
            }
        }

        // todo: init dmabuf, syncobj
        
        Ok(())
    }
}

/// Any error raised by the DRM backend.
#[derive(Debug, Error)]
pub enum DrmBackendError {
    /// An error that occurred while creating a DRM node.
    #[error("could not create drm node: {0}")]
    CreateError(#[from] CreateDrmNodeError),
    /// An error that occured when working with a DRM render node.
    #[error("render node error: {0}")]
    RenderingError(String),
    /// An error occurred while attempting to open a GBM device.
    #[error("unable to open GBM device: {0}")]
    GbmError(#[from] std::io::Error),
    /// An error occurred with the EGL interface.
    #[error("egl error: {0}")]
    EGLError(#[from] SmithayEglError),
    /// An error occurred with the DRM subsystem.
    #[error(transparent)]
    DrmError(#[from] SmithayDrmError),
    #[error("error with DRM device: {0}")]
    DrmDeviceError(String),
    #[error("session error: {0}")]
    SessionError(#[from] LibSeatSessionError),
    #[error("libinput error: {0}")]
    InputError(String),
}

impl WaylandBackendError for DrmBackendError {
    fn backend_name(&self) -> &str {
        "drm"
    }
}

/// An single GPU as handled by DRM.
#[derive(Debug)]
pub struct DrmOutputDevice {
    pub(self) token: RegistrationToken,
    pub(self) render_node: DrmNode,
    pub(self) scanner: DrmScanner,
    ///
    pub(crate) surfaces: HashMap<crtc::Handle, Surface>,
    pub(crate) crtcs: HashMap<crtc::Handle, CrtcInfo>,

    // SAFETY: drop after all the objects used with them are dropped.
    // See https://github.com/Smithay/smithay/issues/1102.
    drm: DrmDevice,
    gbm: GbmDevice<DrmDeviceFd>,

    // todo: DRM leasing
    non_desktop_connectors: HashSet<(connector::Handle, crtc::Handle)>
}


#[derive(Debug, Clone, PartialEq)]
pub struct CrtcInfo {
    id: OutputId,
    name: OutputName,
}

#[derive(Debug)]
pub struct Surface {
    display: DisplayHandle,
    device_id: DrmNode,
    render_node: Option<DrmNode>,
}