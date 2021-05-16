use std::collections::HashSet;

use super::{Ring, Selector};

use crate::x::core::{XWindow, XWindowID};
use crate::core::types::{
    Geometry, Atom,
    WinLayoutState,
    WindowState,
    NetWindowStates,
};

/// A Ring of type Client.
///
/// Contains additional methods more specific to window management.
pub type ClientRing = Ring<Client>;

impl ClientRing {

    pub fn remove_by_id(&mut self, id: XWindowID) -> Option<Client> {
        if let Some(i) = self.get_idx(id) {
            self.remove(i)
        } else {
            None
        }
    }

    pub fn get_idx(&self, id: XWindowID) -> Option<usize> {
        self.index(Selector::Condition(&|win| win.id() == id))
    }

    pub fn lookup(&self, id: XWindowID) -> Option<&Client> {
        if let Some(i) = self.get_idx(id) {
            self.get(i)
        } else {
            None
        }
    }

    pub fn lookup_mut(&mut self, id: XWindowID) -> Option<&mut Client> {
        if let Some(i) = self.get_idx(id) {
            self.get_mut(i)
        } else {
            None
        }
    }

    pub fn contains(&mut self, id: XWindowID) -> bool {
        for win in self.items.iter() {
            if win.id() == id {
                return true
            }
        }
        false
    }

    pub fn set_focused_by_winid(&mut self, id: XWindowID) {
        if let Some(i) = self.get_idx(id) {
            self.focused = Some(i)
        } else {
            //error!("Tried to focus a client not in the workspace")
        }
    }

    pub fn set_focused_by_idx(&mut self, idx: usize) {
        self.set_focused(idx);
    }

    pub fn is_focused(&self, id: XWindowID) -> bool {
        if let Some(window) = self.focused() {
            return window.id() == id
        } else {
            false
        }
    }
}

/// Various metadata about a X window.
/// 
/// Contains ICCCM and EWMH properties.
#[derive(Debug, Clone)]
pub struct Client {
    pub xwindow: XWindow,
    pub name: String,
    pub icon_name: String,
    pub class: (String, String),

    initial_geom: Geometry,
    urgent: bool,
    transient_for: Option<XWindowID>,
    mapped_state: WindowState,
    net_states: NetWindowStates,
    layout_state: WinLayoutState,
    protocols: HashSet<Atom>,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.xwindow.id == other.xwindow.id
    }
}

impl Client {
    pub fn id(&self) -> XWindowID {
        self.xwindow.id
    }
}