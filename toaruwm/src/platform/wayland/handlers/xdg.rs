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
use smithay::desktop::Window;
use smithay::utils::Serial;
use smithay::delegate_xdg_shell;

use crate::platform::wayland::{
    prelude::*,
    window::Unmapped,
};

impl<C: RuntimeConfig, B: WaylandBackend> XdgShellHandler for Wayland<C, B> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.state.xdg_shell
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let wl_surface = surface.wl_surface().clone();
        let unmapped = Unmapped::new(Window::new_wayland_window(surface));

        assert!(self.unmapped.insert(wl_surface, unmapped).is_none());
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {

    }

    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {

    }

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        
    }

    // todo: re-implement provided methods as needed
}

delegate_xdg_shell!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);