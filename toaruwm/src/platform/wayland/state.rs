use smithay::reexports::wayland_server::{
        backend::{ClientId, ClientData, DisconnectReason}, 
        DisplayHandle
    };

use smithay::wayland::{
    compositor::{CompositorState, CompositorClientState}
};

#[derive(Debug)]
pub struct WlState {
    pub(crate) display_handle: DisplayHandle,
    pub(crate) compositor_state: CompositorState,
}

impl WlState {
    fn new() -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct ClientState {
    pub(crate) compositor_state: CompositorClientState
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}