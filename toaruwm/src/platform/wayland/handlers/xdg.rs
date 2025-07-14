//! Functionality for the XDG* Wayland protocols.

#![allow(unused_variables)] // fixme

use tracing::{warn, error};

use smithay::reexports::{
    wayland_server::protocol::{wl_seat::WlSeat, wl_output::WlOutput},
    wayland_protocols::xdg::{
        decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode,
        shell::server::xdg_toplevel,
    },
};
use smithay::wayland::{
    shell::xdg::{
        XdgShellHandler, XdgShellState, ToplevelSurface, PopupSurface, PositionerState,
        decoration::XdgDecorationHandler,
        dialog::XdgDialogHandler
    },
    xdg_foreign::{XdgForeignHandler, XdgForeignState}
};
use smithay::desktop::{Window, PopupKind};
use smithay::utils::Serial;
use smithay::{delegate_xdg_decoration, delegate_xdg_shell, delegate_xdg_dialog, delegate_xdg_foreign};

use crate::platform::wayland::{
    prelude::*,
    window::Unmapped,
};

impl<C: RuntimeConfig, B: WaylandBackend> XdgShellHandler for Wayland<C, B> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState { // done
        &mut self.state_mut().xdg_shell
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) { // done
        let wl_surface = surface.wl_surface().clone();
        let unmapped = Unmapped::new(Window::new_wayland_window(surface));

        assert!(self.wl.unmapped.insert(wl_surface, unmapped).is_none());
    }

    fn new_popup(&mut self, surface: PopupSurface, _: PositionerState) { // done
        let popupkind = PopupKind::Xdg(surface);
        self.unconstrain_popup(&popupkind);

        if let Err(e) = self.wl.popups.track_popup(popupkind) {
            warn!("dead resource while configuring popup: {e}")
        }
    }

    fn move_request(&mut self, surface: ToplevelSurface, seat: WlSeat, serial: Serial) {
        todo!()
    }

    fn resize_request(
        &mut self, 
        surface: ToplevelSurface, 
        seat: WlSeat,
        serial: Serial,
        edges: xdg_toplevel::ResizeEdge,
    ) {
        todo!()
    }

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        todo!()
    }

    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {
        todo!()
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {
        todo!()
    }
    
    fn minimize_request(&mut self, surface: ToplevelSurface) {
        todo!()
    }

    fn fullscreen_request(&mut self, surface: ToplevelSurface, output: Option<WlOutput>) {
        todo!()
    }

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        todo!()
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        todo!()
    }

    fn popup_destroyed(&mut self, surface: PopupSurface) {
        todo!()
    }

    fn app_id_changed(&mut self, surface: ToplevelSurface) {
        todo!()
    }

    fn title_changed(&mut self, surface: ToplevelSurface) {
        todo!()
    }

    fn parent_changed(&mut self, surface: ToplevelSurface) {
        todo!()
    }
}

delegate_xdg_shell!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

impl<C: RuntimeConfig, B: WaylandBackend> XdgDecorationHandler for Wayland<C, B> {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        todo!()
    }

    fn request_mode(&mut self, toplevel: ToplevelSurface, mode: Mode) {
        todo!()
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        todo!()
    }
}

delegate_xdg_decoration!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

impl<C: RuntimeConfig, B: WaylandBackend> XdgDialogHandler for Wayland<C, B> {
    fn modal_changed(&mut self, toplevel: ToplevelSurface, is_modal: bool) {
        todo!()
    }
}

delegate_xdg_dialog!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

impl<C: RuntimeConfig, B: WaylandBackend> XdgForeignHandler for Wayland<C, B> {
    fn xdg_foreign_state(&mut self) -> &mut XdgForeignState {
        &mut self.wl.state.xdg_foreign
    }
}

delegate_xdg_foreign!(@<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> Wayland<C, B>);

#[doc(hidden)]
impl<C: RuntimeConfig, B: WaylandBackend> Wayland<C, B> {
    /// When a top-level is about to be mapped, this is called to 
    /// send the initial configure event.
    pub fn send_initial_configure(&mut self, toplevel: &ToplevelSurface) {
        let Some(unmapped) = self.wl.unmapped.get_mut(toplevel.wl_surface()) else {
            error!("window must not be already configured in send_initial_configure");
            return;
        };

        // todo

        toplevel.send_configure();
    }

    /// Queues the initial configure to be sent when the event loop is idle,
    /// to make sure the client has sent all the info it wants to send.
    pub fn queue_initial_configure(&self, toplevel: ToplevelSurface) {
        self.wl.event_loop.insert_idle(move |wayland| {
            if !toplevel.alive() {
                return
            }

            if let Some(unmapped) = wayland.wl.unmapped.get(toplevel.wl_surface()) {
                if unmapped.needs_initial_configure() {
                    wayland.send_initial_configure(&toplevel);
                }
            }
        });
    }

    /// Unconstrains an XDG popup (see `xdg_positioner.constraint_adjustment`).
    pub fn unconstrain_popup(&self, popup: &PopupKind) {
        // todo
    }
}