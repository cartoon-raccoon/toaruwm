#![allow(unused_variables)]
use crate::x::{
    XEvent, XWindowID, XConn,
    event::{
        ConfigureRequestData,
        ClientMessageEvent,
        PropertyEvent,
    }
};
use crate::core::types::{Geometry, Point};
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
    /// Reconfigure the root window.
    ScreenReconfigure,
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
    RunKeybind(Keybind, XWindowID),
    /// Run the specified mousebind.
    RunMousebind(Mousebind, XWindowID, Point),
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
                debug!("Configure notify");
                if event.id == state.root {
                    vec![ScreenReconfigure]
                } else {
                    vec![]
                }
            }
            ConfigureRequest(event) => {
                debug!("Configure request");
                process_config_request(event, state)
            },
            MapRequest(id, override_redirect) => {
                debug!("Map request for window {}", id);
                process_map_request(id, override_redirect, state)
            },
            MapNotify(_id) => {
                debug!("Map notify for window {}", _id);
                vec![] //* ideally, tell the WM to validate
            }
            UnmapNotify(id) => {
                debug!("Unmap notify for window {}", id);
                vec![UnmapClient(id)]
            }
            DestroyNotify(id) => {
                debug!("Destroy notify for window {}", id);
                vec![DestroyClient(id)]
            },
            // if pointer is not grabbed, tell WM to focus on client
            EnterNotify(id, grab) => {
                debug!("Enter notify for window {}; grab: {}", id, grab);
                if !grab {
                    vec![ClientFocus(id)]
                } else {
                    vec![]
                }
            },
            LeaveNotify(id, grab) => {
                debug!("Leave notify for window {}; grab: {}", id, grab);
                if !grab {
                    vec![ClientUnfocus(id)]
                } else {
                    vec![]
                }
            },
            // This doesn't do anything for now
            ReparentNotify(_event) => {
                debug!("Reparent notify for window {}", _event.child);
                vec![]
            },
            PropertyNotify(event) => {
                debug!("Property notify for window {}", event.id);
                process_property_notify(event, state)
            },
            KeyPress(id, event) => {
                debug!("Keypress notify for window {}", id);
                vec![RunKeybind(event.into(), id)]
            },
            KeyRelease => {
                debug!("Key release notify");
                vec![]
            },
            MouseEvent(event) => {
                debug!("Mouse event for window {}", event.id);
                vec![RunMousebind(
                    event.state, event.id, event.location
                )]
            },
            ClientMessage(event) => {
                info!("Client message received: {:#?}", event);
                process_client_message(event, state)
            },
            Unknown(smth) => {
                info!("Unrecognised event: code {}", smth);
                vec![]
            },
        }
    }
}

#[allow(unused_imports, unused_variables)]
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

fn process_config_request<X: XConn>(
    event: ConfigureRequestData, state: WMState<'_, X>
) -> Vec<EventAction> {
    //todo
    vec![]
}

fn process_property_notify<X: XConn>(
    event: PropertyEvent, state: WMState<'_, X>
) -> Vec<EventAction> {
    //todo
    vec![]
}

fn process_client_message<X: XConn>(
    event: ClientMessageEvent, state: WMState<'_, X>
) -> Vec<EventAction> {
    //todo: this is used extensively in EWMH
    //but is not important to basic operations
    //so we can ignore this for now
    vec![]
}