use smithay::backend::{
    renderer::gles::GlesRenderer,
    winit::{self, WinitGraphicsBackend, WinitEventLoop}
};

use super::WaylandBackend;

#[derive(Debug)]
pub struct WinitBackend {
    pub(crate) winit: WinitGraphicsBackend<GlesRenderer>,
    pub(crate) eventloop: WinitEventLoop,
}

impl WinitBackend {
    pub fn new() -> Self {
        let (winit, eventloop) = winit::init().unwrap(); // fixme

        Self {
            winit,
            eventloop,
        }
    }
}

impl WaylandBackend for WinitBackend {
    fn name(&self) -> &str {
        "winit"
    }

    fn render(&mut self) {

    }
}