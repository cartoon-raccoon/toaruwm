use std::str::FromStr;

use tracing::{debug, info};

use crate::bindings::{Keybind, Mousebind};
use crate::core::types::{Point, Physical};
use crate::manager::{RuntimeConfig, ToaruState};
use crate::platform::x::{
    event::{
        ClientMessageData, ClientMessageEvent, ConfigureRequestData, PointerEvent, PropertyEvent,
    },
    Atom, Property, WmHintsFlags, XAtom, XConn, XError, XEvent, XWindowID,
};
use crate::platform::Platform;

// todo: update as neccesary to account for ICCCM and EWMH conventions
/// Actions that should be taken by the `WindowManager`.
///
/// These are automatically translated within the `WindowManager`
/// from [`XEvent`]s, and you generally shouldn't have to use this
/// directly.
#[derive(Debug, Clone)]
pub enum EventAction<P: Platform> {
    /// Move focus to the specified client.
    MoveClientFocus(P::Client),
    /// Change the WM_NAME property of the specified client.
    ClientNameChange(P::Client),
    /// Detect screens and reconfigure layout.
    ScreenReconfigure,
    /// Set the screen currently in focus from a point.
    ///
    /// If point is None set based on cursor location.
    SetFocusedScreen(Option<Point<Physical>>),
    /// Destroy the specified client.
    DestroyClient(P::Client),
    /// Map the specified client and track it internally.
    ///
    /// Applies to normal top-level windows.
    MapTrackedClient(P::Client),
    /// Map the specified client and manage it without tracking.
    MapUntrackedClient(P::Client),
    /// Unmap the specified client.
    UnmapClient(P::Client),
    /// Configure the specified client with the given geometry.
    ConfigureClient(ConfigureRequestData),
    /// Send the client to the specified workspace.
    ClientToWorkspace(P::Client, usize),
    /// Run the specified keybind.
    RunKeybind(Keybind, P::Client),
    /// Run the specified mousebind.
    RunMousebind(Mousebind, P::Client, Point<Physical>),
    /// Toggle the client in or out of fullscreen.
    ///
    /// Also toggles _NET_WM_STATE_FULLSCREEN.
    ToggleClientFullscreen(P::Client, bool),
    /// Set the state of a window to urgent.
    ToggleUrgency(P::Client),
    /// Handle an error caused by a certain X event.
    HandleError(P::Error, XEvent),
}

impl<P: Platform> EventAction<P> {
    pub(crate) fn from_xevent<C>(
        event: XEvent,
        state: ToaruState<'_, P, C>,
    ) -> Option<Vec<EventAction<P>>>
    where
        C: RuntimeConfig,
    {
        // todo
        None
    }
}

fn process_map_request<P: Platform, C: RuntimeConfig>(
    id: P::Client,
    ovrd: bool,
    state: ToaruState<'_, P, C>,
) -> Option<Vec<EventAction<P>>> {
    use EventAction::*;

    // if window is override-redirect or we already have the window,
    // ignore the request.
    // if ovrd || state.desktop.is_managing(id) {
    //     return None;
    // }

    // if !state.conn.should_manage(id) {
    //     return Some(vec![MapUntrackedClient(id)]);
    // }

    // Some(vec![MapTrackedClient(id)])
    None
}

fn process_enter_notify<P: Platform, C: RuntimeConfig>(
    ptrev: PointerEvent,
    state: ToaruState<'_, P, C>,
) -> Option<Vec<EventAction<P>>> {
    // use EventAction::*;

    // let mut actions = vec![MoveClientFocus(ptrev.id), SetFocusedScreen(Some(ptrev.abs))];

    // if let Some(_focused) = state.desktop.current_client() {
    //     //fixme: is this necessary?
    //     // if next client is set to urgent, unset its urgent flag
    //     if let Some(c) = state.lookup_client(ptrev.id) {
    //         if c.is_urgent() {
    //             actions.push(ToggleUrgency(ptrev.id));
    //         }
    //     }
    // }

    // Some(actions)
    None
}

fn process_property_notify<P: Platform, C: RuntimeConfig>(
    event: PropertyEvent,
    state: ToaruState<'_, P, C>,
) -> Option<Vec<EventAction<P>>> {
    // use EventAction::*;

    // let atom = if let Ok(atom) = state.conn.lookup_atom(event.atom) {
    //     atom
    // } else {
    //     return None;
    // };

    // let hints = Atom::WmHints.as_ref();

    // if !event.deleted && atom == hints {
    //     let wmhints = if let Ok(Some(h)) = state.conn.get_property(hints, event.id) {
    //         h
    //     } else {
    //         return None;
    //     };

    //     if let Property::WMHints(wmhints) = wmhints {
    //         if wmhints.is_set(WmHintsFlags::URGENCY_HINT) {
    //             return Some(vec![ToggleUrgency(event.id)]);
    //         }
    //     }
    // }
    None
}

fn process_client_message<P: Platform, C: RuntimeConfig>(
    event: ClientMessageEvent,
    state: ToaruState<'_, P, C>,
) -> Option<Vec<EventAction<P>>> {
    // use EventAction::*;

    // let is_fullscreen = |data: &[u32]| {
    //     data.iter()
    //         .flat_map(|&a| state.conn.lookup_atom(XAtom::from(a)))
    //         .any(|s| s == Atom::NetWmStateFullscreen.as_ref())
    // };

    // let atom = match state.conn.lookup_atom(event.type_) {
    //     Ok(atom) => atom,
    //     Err(_) => return None,
    // };

    // if let ClientMessageData::U32(data) = event.data {
    //     match Atom::from_str(&atom) {
    //         Ok(Atom::NetActiveWindow) => None, //todo
    //         Ok(Atom::NetWmDesktop) => Some(vec![ClientToWorkspace(event.window, data[0] as usize)]),
    //         Ok(Atom::NetWmState) if is_fullscreen(&data[1..3]) => {
    //             let should_fullscreen = [1, 2].contains(&data[0]);

    //             Some(vec![ToggleClientFullscreen(
    //                 event.window,
    //                 should_fullscreen,
    //             )])
    //         }
    //         _ => {
    //             debug!(target: "manager::event","Got client message of type {}, data {:?}", atom, data);
    //             None
    //         }
    //     }
    // } else {
    //     debug!(target: "manager::event","Got client message of type {}, data {:?}", atom, event.data);
    //     None
    // }
    None
}
