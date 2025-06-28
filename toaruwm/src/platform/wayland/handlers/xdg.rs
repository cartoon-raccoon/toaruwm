//! XDG protocol functionality.

use smithay::reexports::{
    wayland_server::protocol::wl_seat::WlSeat,
};
use smithay::wayland::{
    shell::xdg::{
        XdgShellHandler, XdgShellState, ToplevelSurface,
        PopupSurface, PositionerState,
    },
};
use smithay::utils::Serial;
use smithay::delegate_xdg_shell;

use crate::platform::wayland::{Wayland, backend::WaylandBackend};
use crate::manager::RuntimeConfig;

impl<C: RuntimeConfig, B: WaylandBackend> XdgShellHandler for Wayland<C, B> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.state.xdg_shell
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {

    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {

    }

    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {

    }

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        
    }
}

delegate_xdg_shell!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);