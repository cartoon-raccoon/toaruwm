use crate::x::{
    XEvent, XWindowID, 
    XConn, XError,
    event::{
        ConfigureRequestData,
        ClientMessageEvent,
        PropertyEvent,
        PointerEvent,
    },
};
use crate::core::types::{Geometry, Point};
use crate::keybinds::{Keybind, Mousebind};
use crate::manager::WMState;

// todo: update as neccesary to account for ICCCM and EWMH conventions
#[derive(Debug, Clone)]
pub enum EventAction {
    /// Focus the specified client.
    ClientFocus(XWindowID),
    /// Unfocus the specified client.
    ClientUnfocus(XWindowID),
    /// Change the WM_NAME property of the specified client.
    ClientNameChange(XWindowID),
    /// Detect screens and reconfigure layout.
    ScreenReconfigure,
    /// Set the screen currently in focus from a point.
    /// 
    /// If point is None set based on cursor location.
    SetFocusedScreen(Option<Point>),
    /// Destroy the specified client.
    DestroyClient(XWindowID),
    /// Map the specified client and track it internally.
    /// 
    /// Applies to normal top-level windows.
    MapTrackedClient(XWindowID),
    /// Map the specified client and manage it without tracking.
    MapUntrackedClient(XWindowID),
    /// Unmap the specified client.
    UnmapClient(XWindowID),
    /// Configure the specified client with the given geometry.
    ConfigureClient(ConfigureRequestData),
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
    /// Handle an error caused by a certain X event.
    HandleError(XError, XEvent),
}

impl EventAction {
    
    pub(crate) fn from_xevent<X: XConn>(event: XEvent, state: WMState<'_, X>) -> Vec<EventAction> {
        use EventAction::*;
        use XEvent::*;
        match event {
            ConfigureNotify(event) => {
                debug!("Configure notify");
                if event.id == state.root.id {
                    vec![ScreenReconfigure]
                } else {
                    vec![]
                }
            }
            ConfigureRequest(event) => {
                debug!("Configure request for window {}", event.id);
                vec![ConfigureClient(event)]
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
            EnterNotify(ev, grab) => {
                debug!("Enter notify for window {}; grab: {}", ev.id, grab);
                if !grab {
                    process_enter_notify(ev, state)
                } else {
                    vec![]
                }
            },
            LeaveNotify(ev, grab) => {
                debug!("Leave notify for window {}; grab: {}", ev.id, grab);
                if !grab {
                    vec![ClientUnfocus(ev.id), SetFocusedScreen(Some(ev.abs))]
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
            RandrNotify => vec![ScreenReconfigure],
            ScreenChange => vec![SetFocusedScreen(None)],
            Unknown(smth) => {
                info!("Unrecognised event: code {}", smth);
                vec![]
            },
        }
    }
}

fn process_map_request<X: XConn>(
    id: XWindowID, ovrd: bool, state: WMState<'_, X>
) -> Vec<EventAction> {
    use EventAction::*;

    // if window is override-redirect or we already have the window,
    // ignore the request.
    if ovrd || state.desktop.is_managing(id) {
        return Vec::new()
    }

    if !state.conn.should_manage(id) {
        return vec![MapUntrackedClient(id)]
    }

    vec![MapTrackedClient(id)]
}

fn process_enter_notify<X: XConn>(
   pt: PointerEvent, state: WMState<'_, X>
) -> Vec<EventAction> {
    use EventAction::*;

    let mut actions = vec![ClientFocus(pt.id), SetFocusedScreen(Some(pt.abs))];

    if let Some(focused) = state.focused {
        if focused != pt.id {
            actions.insert(0, ClientUnfocus(focused))
        }
    }

    actions
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