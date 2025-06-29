use tracing::trace;

use std::collections::hash_map::Entry;

use smithay::reexports::{
    wayland_server::{
        protocol::wl_surface::WlSurface,
        Client as WlClient, Resource,
    }
};
use smithay::wayland::{
    compositor::{
        CompositorState, CompositorClientState, CompositorHandler,
        is_sync_subsurface, get_parent,
    }
};
use smithay::backend::{
    renderer::utils::on_commit_buffer_handler,
};

use smithay::delegate_compositor;

use crate::platform::wayland::{
    prelude::*,
    state::ClientState,
    window::is_mapped,
};

impl<C, B> CompositorHandler for Wayland<C, B>
where
    C: RuntimeConfig + 'static,
    B: WaylandBackend + 'static
{
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.state.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a WlClient) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        trace!(surface = ?surface.id(), "commit");

        on_commit_buffer_handler::<Self>(surface);
        self.backend.early_import(surface);

        if is_sync_subsurface(surface) { 
            return
        }

        let mut root = surface.clone();
        while let Some(parent) = get_parent(&root) {
            root = parent;
        }

        self.root_surfaces.insert(surface.clone(), root.clone());

        // this is a root surface commit that might have mapped a previously-unmapped toplevel.
        if surface == &root {
            // the toplevel is currently unmapped in our state.
            if let Entry::Occupied(entry) = self.unmapped.entry(surface.clone()) {
                if is_mapped(surface) {
                    // the toplevel got mapped.
                } else {
                    // the toplevel remains unmapped.
                }
            } else /* find window and corresponding output */ {
            // this is a commit of a previously unmapped root or a non-toplevel root.

            }
        } else { // this is a commit of a non-root or non-toplevel root.
        }
        // todo
        // get all outputs the surface appears on, and regen elements, then call render
    }
}

delegate_compositor!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);