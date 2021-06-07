use crate::x::{XEvent, XWindowID, XConn};
use crate::core::types::Geometry;
use crate::keybinds::{Keybind, Mousebind};
use crate::manager::WMState;

// todo: update as neccesary to account for ICCCM and EWMH conventions
pub enum EventAction {
    /// Focus the specified client.
    ClientFocus(XWindowID),
    /// Unfocus the specified client.
    ClientUnfocus(XWindowID),
    /// Change the WM_NAME property of the specified client.
    ClientNameChange(XWindowID),
    /// Destroy the specified client.
    DestroyClient(XWindowID),
    /// Map the specified client and track it internally.
    /// 
    /// Applies to normal windows.
    MapTrackedClient(XWindowID),
    /// Map the specified client and manage it without tracking.
    /// 
    /// Used for dialogue boxes and other windows that have
    /// WM_TRANSIENT_FOR set.
    MapUntrackedClient(XWindowID),
    /// Unmap the specified client.
    UnmapClient(XWindowID),
    /// Configure the specified client with the given geometry.
    ConfigureClient(XWindowID, Geometry),
    /// Run the specified keybind.
    RunKeybind(Keybind),
    /// Run the specified mousebind.
    RunMousebind(Mousebind),
    /// Toggle the client in or out of fullscreen.
    /// 
    /// Also toggles _NET_WM_STATE_FULLSCREEN.
    ToggleClientFullscreen(XWindowID, bool),
    /// Set the state of a window to urgent.
    ToggleUrgency(XWindowID),
}

impl EventAction {
    
    #[allow(unused_imports, dead_code, unused_variables)]
    pub(crate) fn from_xevent<X: XConn>(event: XEvent, state: WMState<'_, X>) -> Vec<EventAction> {
        use EventAction::*;
        use XEvent::*;
        match event {
            ConfigureNotify(event) => {

            }
            ConfigureRequest(event) => {

            },
            MapRequest(id, override_redirect) => {

            },
            MapNotify(id) => {}
            UnmapNotify(id) => {}
            DestroyNotify(id) => {},
            EnterNotify(id) => {},
            LeaveNotify(id) => {},
            MotionNotify(id, pt) => {},
            ReparentNotify(id) => {},
            PropertyNotify(event) => {},
            KeyPress(id, event) => {},
            KeyRelease => {},
            ButtonPress(event) => {},
            ButtonRelease => {},
            ClientMessage(event) => {},
            Unknown(smth) => {},
        }
        todo!()
    }
}

#[allow(unused_imports, dead_code, unused_variables)]
fn process_map_request<X: XConn>(
    id: XWindowID, ovrd: bool, state: WMState<'_, X>
) -> Vec<EventAction> {
    use EventAction::*;
    use XEvent::*;
    
    // if let Some(window_type) = state.conn.get_window_type(id) {
    //     let atoms = state.conn.get_atoms();


    // }

    todo!("process_map_request")
}