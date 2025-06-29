use smithay::reexports::{
    wayland_server::protocol::wl_output::WlOutput
};
use smithay::wayland::{
    shell::{
        wlr_layer::{WlrLayerShellHandler, WlrLayerShellState, Layer, LayerSurface}
    }
};
use smithay::delegate_layer_shell;

use crate::platform::wayland::prelude::*;

impl<C: RuntimeConfig, B: WaylandBackend> WlrLayerShellHandler for Wayland<C, B> {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.state_mut().layer_shell
    }

    fn new_layer_surface(&mut self, 
        surface: LayerSurface, 
        output: Option<WlOutput>,
        layer: Layer,
        namespace: String,
    ) {

    }
}

delegate_layer_shell!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);