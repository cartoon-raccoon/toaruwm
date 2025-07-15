use tracing::warn;

use std::marker::PhantomData;

use smithay::backend::renderer::{ImportDma, ImportEgl};
use smithay::reexports::{
    wayland_server::DisplayHandle,
};
use smithay::backend::{
    renderer::{
        gles::GlesRenderer,
        damage::OutputDamageTracker,
    },
    winit::{
        self,
        WinitGraphicsBackend, WinitEventLoop, WinitEvent,
        Error as WinitError,
    },
    allocator::{
        dmabuf::Dmabuf,
    }
};
use smithay::output::{
    Mode, Output, PhysicalProperties, Subpixel,
};
use smithay::utils::Transform;

use tracing::{error};

use thiserror::Error;

use super::{WaylandBackendError};
use crate::platform::wayland::{prelude::*, WaylandImpl, WaylandError};
use crate::types::Dict;
use crate::dict;

const OUTPUT_NAME: &str = "winit";

#[derive(Debug)]
pub struct WinitBackend<M: Manager<Wayland<M, Self>> + 'static> {
    pub(crate) winit: WinitGraphicsBackend<GlesRenderer>,
    pub(crate) dmg_tracker: OutputDamageTracker,
    pub(crate) output: Output,

    _phantom: PhantomData<M>,
}

impl<M: Manager<Wayland<M, Self>> + 'static> WinitBackend<M> {
    /// Creates a new Winit backend, returning additional args inside a `Dict`
    /// that must be passed into its `init` method.
    pub fn new() -> Result<(Self, Dict), WaylandError> {
        let (winit, eventloop) = winit::init()  
            .map_err(|e| WinitBackendError::from(e))?;

        let size = winit.window_size();
        let mode = Mode {
            size,
            refresh: 60_000,
        };

        let output = Output::new(
            OUTPUT_NAME.to_string(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "Toaru".into(),
                model: "Winit".into(),
            }
        );

        output.change_current_state(Some(mode), Some(Transform::Flipped180), None, Some((0, 0).into()));
        output.set_preferred(mode);

        let dmg_tracker = OutputDamageTracker::from_output(&output);

        let args = dict! {
            "winitev" => eventloop,
        };

        Ok((Self {
            winit,
            dmg_tracker,
            output,
            _phantom: PhantomData,
        }, args))
    }
}

impl<M: Manager<Wayland<M, Self>> + 'static> WaylandBackend<M> for WinitBackend<M> {
    
    fn name(&self) -> &str {
        "winit"
    }
    fn nested(&self) -> bool {
        true
    }

    fn seat_name(&self) -> &str {
        "winit"
    }

    fn render(&mut self, wl: &mut WaylandImpl<M, Self>)
    where
        Self: Sized
    {

    }

    fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        match self.winit.renderer().import_dmabuf(dmabuf, None) {
            Ok(_txtr) => true,
            Err(e) => {
                error!("error while importing DMA-BUF: {e}");
                false
            }
        }
    }

    fn init(
        &mut self, 
        display: DisplayHandle,
        wl_impl: &mut WaylandImpl<M, Self>,
        mut args: Dict)-> Result<(), WaylandError> {
        let winitev = args.remove("winitev")
            .and_then(|wev| wev.downcast::<WinitEventLoop>().ok())
            .expect("error in initializing WinitBackend: no Winit Event Loop was provided");

        let renderer = self.winit.renderer();
        if let Err(err) = renderer.bind_wl_display(&display) {
            warn!("error binding display to renderer: {err}");
        }

        wl_impl.event_loop.insert_source(winitev, |event, _, wayland| {
            match event {
                WinitEvent::Resized {size, scale_factor} => {

                }
                WinitEvent::Focus(_) => {}
                WinitEvent::Input(ievent) => wayland.handle_input_event(ievent),
                WinitEvent::Redraw => { /* todo */}
                WinitEvent::CloseRequested => wayland.wl.stop_signal.stop(),
            }
        }).map_err(|e| e.error)?;

        // todo: create dma-buf default feedback
        
        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("winit backend error: {src}")]
pub struct WinitBackendError {
    #[source]
    #[from]
    src: WinitError,
}

impl WaylandBackendError for WinitBackendError {
    fn backend_name(&self) -> &str {
        "winit"
    }
}