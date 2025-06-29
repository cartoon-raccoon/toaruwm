use smithay::reexports::{
    wayland_server::{
        backend::{ClientId, ClientData, DisconnectReason}, 
        DisplayHandle,
        protocol::{
            wl_buffer::WlBuffer,
            wl_surface::WlSurface,
        },
    }
};

use smithay::wayland::{
    compositor::{CompositorState, CompositorClientState},
    shell::{
        xdg::{XdgShellState},
        wlr_layer::{WlrLayerShellState},
    },
    shm::{ShmHandler, ShmState},
    buffer::{BufferHandler},
    dmabuf::{DmabufState, DmabufHandler, DmabufGlobal, ImportNotifier},
};
use smithay::input::{
    SeatState, SeatHandler
};
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::{delegate_shm, delegate_dmabuf, delegate_seat};

use crate::platform::wayland::prelude::*;

/// The compositor state.
/// 
/// `WlState` stores all Smithay-related `-State` types.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct WlState<C, B>
where
    C: RuntimeConfig + 'static,
    B: WaylandBackend + 'static,
{
    pub display_handle: DisplayHandle,
    pub compositor_state: CompositorState,
    pub xdg_shell: XdgShellState,
    pub layer_shell: WlrLayerShellState,

    pub shm_state: ShmState,
    pub dmabuf_state: DmabufState,
    pub seat_state: SeatState<Wayland<C, B>>,
}

impl<C: RuntimeConfig, B: WaylandBackend> WlState<C, B> {
    pub fn new(display_handle: DisplayHandle) -> Self
    where
        C: RuntimeConfig,
        B: WaylandBackend
    {
        let compositor_state = CompositorState::new::<Wayland<C, B>>(&display_handle);
        let xdg_shell = XdgShellState::new::<Wayland<C, B>>(&display_handle);
        let layer_shell = WlrLayerShellState::new::<Wayland<C, B>>(&display_handle);
        let shm_state = ShmState::new::<Wayland<C, B>>(&display_handle, vec![]);
        let dmabuf_state = DmabufState::new();
        let seat_state = SeatState::new();

        Self {
            display_handle,
            compositor_state,
            xdg_shell,
            layer_shell,
            shm_state,
            dmabuf_state,
            seat_state,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct ClientState {
    /// Per-client compositor state
    pub(crate) compositor_state: CompositorClientState,
    /// Whether this client is restricted from security-sensitive protocols.
    pub(crate) restricted: bool,
    /// Whether we know this client's security credentials.
    pub(crate) credentials_unknown: bool,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

// handler delegation for buffer

impl<C: RuntimeConfig, B: WaylandBackend> BufferHandler for Wayland<C, B> {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl<C: RuntimeConfig, B: WaylandBackend> ShmHandler for Wayland<C, B> {
    fn shm_state(&self) -> &ShmState {
        &self.state.shm_state
    }
}

delegate_shm!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

impl<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> DmabufHandler for Wayland<C, B> {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.state.dmabuf_state
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, dmabuf: Dmabuf, notifier: ImportNotifier) {
        if self.backend.import_dmabuf(&dmabuf) {
            let _ = notifier.successful::<Wayland<C, B>>();
        } else {
            notifier.failed();
        }
    }
}

delegate_dmabuf!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

impl<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> SeatHandler for Wayland<C, B> {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.state.seat_state
    }

    // todo: reimplement provided methods as needed
}

delegate_seat!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

