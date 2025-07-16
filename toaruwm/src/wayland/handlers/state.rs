use std::sync::Arc;
use std::os::unix::prelude::OwnedFd;

use smithay::reexports::{
    wayland_server::{
        backend::{ ClientData, ClientId, DisconnectReason }, 
        protocol::{
            wl_buffer::WlBuffer, wl_output::WlOutput,
        },
        DisplayHandle
    }
};
use smithay::wayland::{
    compositor::{ CompositorState, CompositorClientState },
    selection::{
        SelectionHandler, SelectionTarget,
        data_device::{
            DataDeviceState, DataDeviceHandler, ServerDndGrabHandler, ClientDndGrabHandler
        },
    },
    shell::{
        xdg::{XdgShellState, decoration::XdgDecorationState, dialog::XdgDialogState},
        wlr_layer::{WlrLayerShellState},
    },
    output::{
        OutputManagerState, OutputHandler,
    },
    shm::{ShmHandler, ShmState},
    buffer::{BufferHandler},
    dmabuf::{DmabufState, DmabufHandler, DmabufGlobal, ImportNotifier},
    pointer_gestures::PointerGesturesState,
    relative_pointer::RelativePointerManagerState,
    xdg_foreign::XdgForeignState,
};
use smithay::input::{
    Seat, SeatState, SeatHandler
};
use smithay::output::Output;
use smithay::backend::allocator::dmabuf::Dmabuf;

use crate::{
    delegate_shm, delegate_dmabuf, delegate_output, delegate_seat, delegate_data_device,
    delegate_pointer_gestures, delegate_relative_pointer
};
use crate::wayland::{
    prelude::*, input::{KeyboardFocusTarget, PointerFocusTarget},
};

/// The compositor state.
/// 
/// `WlState` stores all Smithay-related `-State` types.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct WaylandState<M, B>
where
    M: Manager + 'static,
    B: WaylandBackend<M> + 'static,
{
    pub display_handle: DisplayHandle,

    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub output_manager_state: OutputManagerState,

    pub pointer_gestures: PointerGesturesState,
    pub relative_pointer: RelativePointerManagerState,

    pub xdg_shell: XdgShellState,
    pub xdg_decoration: XdgDecorationState,
    pub xdg_dialog: XdgDialogState,
    pub xdg_foreign: XdgForeignState,
    pub layer_shell: WlrLayerShellState,

    pub shm_state: ShmState,
    pub dmabuf_state: DmabufState,
    pub seat_state: SeatState<Wayland<M, B>>,
}

impl<M: Manager, B: WaylandBackend<M>> WaylandState<M, B> {
    pub fn new(display_handle: DisplayHandle) -> Self {
        let compositor_state = CompositorState::new::<Wayland<M, B>>(&display_handle);
        let data_device_state = DataDeviceState::new::<Wayland<M, B>>(&display_handle);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Wayland<M, B>>(&display_handle);
        let pointer_gestures = PointerGesturesState::new::<Wayland<M, B>>(&display_handle);
        let relative_pointer = RelativePointerManagerState::new::<Wayland<M, B>>(&display_handle);
        let xdg_shell = XdgShellState::new::<Wayland<M, B>>(&display_handle);
        let xdg_decoration = XdgDecorationState::new::<Wayland<M, B>>(&display_handle);
        let xdg_dialog = XdgDialogState::new::<Wayland<M, B>>(&display_handle);
        let xdg_foreign = XdgForeignState::new::<Wayland<M, B>>(&display_handle);
        let layer_shell = WlrLayerShellState::new::<Wayland<M, B>>(&display_handle);
        let shm_state = ShmState::new::<Wayland<M, B>>(&display_handle, vec![]);
        let dmabuf_state = DmabufState::new();
        let seat_state = SeatState::new();

        Self {
            display_handle,

            compositor_state,
            data_device_state,
            output_manager_state,
            pointer_gestures,
            relative_pointer,

            xdg_shell,
            xdg_decoration,
            xdg_dialog,
            xdg_foreign,
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

impl<M: Manager, B: WaylandBackend<M>> BufferHandler for Wayland<M, B> {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl<M: Manager, B: WaylandBackend<M>> ShmHandler for Wayland<M, B> {
    fn shm_state(&self) -> &ShmState {
        &self.state().shm_state
    }
}

delegate_shm!(@<M: Manager + 'static, B: WaylandBackend<M> + 'static> Wayland<M, B>);

impl<M: Manager + 'static, B: WaylandBackend<M> + 'static> DmabufHandler for Wayland<M, B> {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.state_mut().dmabuf_state
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, dmabuf: Dmabuf, notifier: ImportNotifier) {
        if self.backend.import_dmabuf(&dmabuf) {
            let _ = notifier.successful::<Wayland<M, B>>();
        } else {
            notifier.failed();
        }
    }
}

delegate_dmabuf!(@<M: Manager + 'static, B: WaylandBackend<M> + 'static> Wayland<M, B>);

impl<M: Manager + 'static, B: WaylandBackend<M> + 'static> SeatHandler for Wayland<M, B> {
    type KeyboardFocus = KeyboardFocusTarget;
    type PointerFocus = PointerFocusTarget;
    type TouchFocus = PointerFocusTarget;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.state_mut().seat_state
    }

    // todo: reimplement provided methods as needed
}

impl<M: Manager + 'static, B: WaylandBackend<M> + 'static> OutputHandler for Wayland<M, B> {
    fn output_bound(&mut self, _output: Output, _wl_output: WlOutput) {
        // todo
    }
}

delegate_output!(@<M: Manager + 'static, B: WaylandBackend<M> + 'static> Wayland<M, B>);

delegate_seat!(@<M: Manager + 'static, B: WaylandBackend<M> + 'static> Wayland<M, B>);

impl<M: Manager + 'static, B: WaylandBackend<M> + 'static> ServerDndGrabHandler for Wayland<M, B> {
    // todo maybe?
}

impl<M: Manager + 'static, B: WaylandBackend<M> + 'static> ClientDndGrabHandler for Wayland<M, B> {
    // todo
}

impl<M: Manager + 'static, B: WaylandBackend<M> + 'static> DataDeviceHandler for Wayland<M, B> {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.state().data_device_state
    }
}

delegate_data_device!(@<M: Manager + 'static, B: WaylandBackend<M> + 'static> Wayland<M, B>);

impl<M: Manager + 'static, B: WaylandBackend<M> + 'static> SelectionHandler for Wayland<M, B> {
    type SelectionUserData = Arc<[u8]>;
    
    fn send_selection(
        &mut self,
        ty: SelectionTarget,
        mime_type: String,
        fd: OwnedFd,
        seat: Seat<Self>,
        user_data: &Self::SelectionUserData,
    ) {
        // todo
    }
}

delegate_pointer_gestures!(@<M: Manager + 'static, B: WaylandBackend<M> + 'static> Wayland<M, B>);

delegate_relative_pointer!(@<M: Manager + 'static, B: WaylandBackend<M> + 'static> Wayland<M, B>);

