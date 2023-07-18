use std::str::FromStr;

use tracing::{debug, info};

use crate::bindings::{Keybind, Mousebind};
use crate::core::types::Point;
use crate::manager::{RuntimeConfig, WmState};
use crate::x::{
    event::{
        ClientMessageData, ClientMessageEvent, ConfigureRequestData, PointerEvent, PropertyEvent,
    },
    Atom, Property, WmHintsFlags, XConn, XError, XEvent, XWindowID,
};

// todo: update as neccesary to account for ICCCM and EWMH conventions
/// Actions that should be taken by the `WindowManager`.
///
/// These are automatically translated within the `WindowManager`
/// from [`XEvent`]s, and you generally shouldn't have to use this
/// directly.
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
    /// Send the client to the specified workspace.
    ClientToWorkspace(XWindowID, usize),
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
    pub(crate) fn from_xevent<X, C>(
        event: XEvent,
        state: WmState<'_, X, C>,
    ) -> Option<Vec<EventAction>>
    where
        X: XConn,
        C: RuntimeConfig,
    {
        use EventAction::*;
        use XEvent::*;
        match event {
            ConfigureNotify(event) => {
                debug!(target: "manager::event","configure notify for window {}", event.id);
                if event.id == state.root.id {
                    Some(vec![ScreenReconfigure])
                } else {
                    None
                }
            }
            ConfigureRequest(event) => {
                debug!(target: "manager::event","configure request for window {}", event.id);
                Some(vec![ConfigureClient(event)])
            }
            MapRequest(id, override_redirect) => {
                debug!(target: "manager::event","map request for window {}", id);
                process_map_request(id, override_redirect, state)
            }
            MapNotify(_id, _from_root) => {
                debug!(target: "manager::event","map notify for window {}", _id);
                None //* ideally, tell the WM to validate
            }
            UnmapNotify(id, from_root) => {
                debug!(target: "manager::event","unmap notify for window {}", id);
                if from_root {
                    None
                } else {
                    Some(vec![UnmapClient(id)])
                }
            }
            DestroyNotify(id) => {
                debug!(target: "manager::event","destroy notify for window {}", id);
                Some(vec![DestroyClient(id)])
            }
            // if pointer is not grabbed, tell WM to focus on client
            EnterNotify(ev, grab) => {
                debug!(target: "manager::event","enter notify for window {}; grab: {}", ev.id, grab);
                if !grab && state.is_managing(ev.id) {
                    process_enter_notify(ev, state)
                } else {
                    None
                }
            }
            LeaveNotify(ev, grab) => {
                debug!(target: "manager::event","leave notify for window {}; grab: {}", ev.id, grab);
                if !grab && state.is_managing(ev.id) {
                    Some(vec![ClientUnfocus(ev.id), SetFocusedScreen(Some(ev.abs))])
                } else {
                    None
                }
            }
            // This doesn't do anything for now
            ReparentNotify(_event) => {
                debug!(target: "manager::event","reparent notify for window {}", _event.child);
                None
            }
            PropertyNotify(event) => {
                // ignore if window that changed was the root
                if event.id == state.root.id {
                    return None;
                }
                debug!(target: "manager::event","property notify for window {}", event.id);
                process_property_notify(event, state)
            }
            KeyPress(id, event) => {
                debug!(target: "manager::event","keypress notify for window {}", id);
                Some(vec![RunKeybind(event.into(), id)])
            }
            KeyRelease => {
                debug!(target: "manager::event","key release notify");
                None
            }
            MouseEvent(event) => {
                debug!(target: "manager::event","mouse event for window {}", event.id);
                Some(vec![RunMousebind(event.state, event.id, event.location)])
            }
            ClientMessage(event) => {
                info!("Client message received: {:#?}", event);
                process_client_message(event, state)
            }
            RandrNotify => Some(vec![ScreenReconfigure]),
            ScreenChange => Some(vec![SetFocusedScreen(None)]),
            Unknown(smth) => {
                info!("Unrecognised event: {}", smth);
                None
            }
        }
    }
}

fn process_map_request<X: XConn, C: RuntimeConfig>(
    id: XWindowID,
    ovrd: bool,
    state: WmState<'_, X, C>,
) -> Option<Vec<EventAction>> {
    use EventAction::*;

    // if window is override-redirect or we already have the window,
    // ignore the request.
    if ovrd || state.desktop.is_managing(id) {
        return None;
    }

    if !state.conn.should_manage(id) {
        return Some(vec![MapUntrackedClient(id)]);
    }

    Some(vec![MapTrackedClient(id)])
}

fn process_enter_notify<X: XConn, C: RuntimeConfig>(
    pt: PointerEvent,
    state: WmState<'_, X, C>,
) -> Option<Vec<EventAction>> {
    use EventAction::*;

    let mut actions = vec![ClientFocus(pt.id), SetFocusedScreen(Some(pt.abs))];

    if let Some(focused) = state.desktop.current_client() {
        // unfocus previous client
        if focused.id() != pt.id {
            actions.insert(0, ClientUnfocus(focused.id()))
        }
        // if next client is set to urgent, unset its urgent flag
        if let Some(c) = state.lookup_client(pt.id) {
            if c.is_urgent() {
                actions.push(ToggleUrgency(pt.id));
            }
        }
    }

    Some(actions)
}

fn process_property_notify<X: XConn, C: RuntimeConfig>(
    event: PropertyEvent,
    state: WmState<'_, X, C>,
) -> Option<Vec<EventAction>> {
    use EventAction::*;

    let atom = if let Ok(atom) = state.conn.lookup_atom(event.atom) {
        atom
    } else {
        return None;
    };

    let hints = Atom::WmHints.as_ref();

    if !event.deleted && atom == hints {
        let wmhints = if let Ok(Some(h)) = state.conn.get_property(hints, event.id) {
            h
        } else {
            return None;
        };

        if let Property::WMHints(wmhints) = wmhints {
            if wmhints.is_set(WmHintsFlags::URGENCY_HINT) {
                return Some(vec![ToggleUrgency(event.id)]);
            }
        }
    }
    None
}

fn process_client_message<X: XConn, C: RuntimeConfig>(
    event: ClientMessageEvent,
    state: WmState<'_, X, C>,
) -> Option<Vec<EventAction>> {
    use EventAction::*;

    let is_fullscreen = |data: &[u32]| {
        data.iter()
            .flat_map(|&a| state.conn.lookup_atom(a))
            .any(|s| s == Atom::NetWmStateFullscreen.as_ref())
    };

    let atom = match state.conn.lookup_atom(event.type_) {
        Ok(atom) => atom,
        Err(_) => return None,
    };

    if let ClientMessageData::U32(data) = event.data {
        match Atom::from_str(&atom) {
            Ok(Atom::NetActiveWindow) => None, //todo
            Ok(Atom::NetWmDesktop) => Some(vec![ClientToWorkspace(event.window, data[0] as usize)]),
            Ok(Atom::NetWmState) if is_fullscreen(&data[1..3]) => {
                let should_fullscreen = [1, 2].contains(&data[0]);

                Some(vec![ToggleClientFullscreen(
                    event.window,
                    should_fullscreen,
                )])
            }
            _ => {
                debug!(target: "manager::event","Got client message of type {}, data {:?}", atom, data);
                None
            }
        }
    } else {
        debug!(target: "manager::event","Got client message of type {}, data {:?}", atom, event.data);
        None
    }
}
