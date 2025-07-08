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
use smithay::{
    delegate_shm, delegate_dmabuf, delegate_seat, delegate_output, delegate_pointer_gestures,
    delegate_relative_pointer, delegate_data_device
};

use crate::platform::wayland::{
    prelude::*, input::{KeyboardFocusTarget, PointerFocusTarget},
};

/// The compositor state.
/// 
/// `WlState` stores all Smithay-related `-State` types.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct WaylandState<C, B>
where
    C: RuntimeConfig + 'static,
    B: WaylandBackend + 'static,
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
    pub seat_state: SeatState<Wayland<C, B>>,
}

impl<C: RuntimeConfig, B: WaylandBackend> WaylandState<C, B> {
    pub fn new(display_handle: DisplayHandle) -> Self
    where
        C: RuntimeConfig,
        B: WaylandBackend
    {
        let compositor_state = CompositorState::new::<Wayland<C, B>>(&display_handle);
        let data_device_state = DataDeviceState::new::<Wayland<C, B>>(&display_handle);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Wayland<C, B>>(&display_handle);
        let pointer_gestures = PointerGesturesState::new::<Wayland<C, B>>(&display_handle);
        let relative_pointer = RelativePointerManagerState::new::<Wayland<C, B>>(&display_handle);
        let xdg_shell = XdgShellState::new::<Wayland<C, B>>(&display_handle);
        let xdg_decoration = XdgDecorationState::new::<Wayland<C, B>>(&display_handle);
        let xdg_dialog = XdgDialogState::new::<Wayland<C, B>>(&display_handle);
        let xdg_foreign = XdgForeignState::new::<Wayland<C, B>>(&display_handle);
        let layer_shell = WlrLayerShellState::new::<Wayland<C, B>>(&display_handle);
        let shm_state = ShmState::new::<Wayland<C, B>>(&display_handle, vec![]);
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

impl<C: RuntimeConfig, B: WaylandBackend> BufferHandler for Wayland<C, B> {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl<C: RuntimeConfig, B: WaylandBackend> ShmHandler for Wayland<C, B> {
    fn shm_state(&self) -> &ShmState {
        &self.state().shm_state
    }
}

delegate_shm!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

impl<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> DmabufHandler for Wayland<C, B> {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.state_mut().dmabuf_state
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
    type KeyboardFocus = KeyboardFocusTarget;
    type PointerFocus = PointerFocusTarget;
    type TouchFocus = PointerFocusTarget;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.state_mut().seat_state
    }

    // todo: reimplement provided methods as needed
}

impl<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> OutputHandler for Wayland<C, B> {
    fn output_bound(&mut self, _output: Output, _wl_output: WlOutput) {
        // todo
    }
}

delegate_output!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

delegate_seat!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

impl<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> ServerDndGrabHandler for Wayland<C, B> {
    // todo maybe?
}

impl<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> ClientDndGrabHandler for Wayland<C, B> {
    // todo
}

impl<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> DataDeviceHandler for Wayland<C, B> {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.state().data_device_state
    }
}

delegate_data_device!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

impl<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> SelectionHandler for Wayland<C, B> {
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

delegate_pointer_gestures!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

delegate_relative_pointer!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

