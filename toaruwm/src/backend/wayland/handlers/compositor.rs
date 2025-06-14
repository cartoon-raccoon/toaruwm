use smithay::reexports::{
    wayland_server::{
        protocol::wl_surface::WlSurface,
        Client as WlClient
    }
};

use smithay::wayland::{
    compositor::{CompositorState, CompositorClientState, CompositorHandler}
};

use smithay::delegate_compositor;

use crate::backend::wayland::{WlState, state::ClientState};

impl CompositorHandler for WlState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a WlClient) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        // todo
    }
}

delegate_compositor!(WlState);