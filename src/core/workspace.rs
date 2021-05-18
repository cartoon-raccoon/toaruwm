use crate::core::{
    window::{Client, ClientRing},
    desktop::Screen,
};
use crate::layouts::{LayoutType, LayoutEngine};
use crate::x::{XConn, XWindowID};

pub struct Workspace {
    pub(crate) windows: ClientRing,
    pub(crate) master: Option<XWindowID>,
    pub(crate) layoutter: LayoutEngine,

    _activate: fn(&dyn XConn, &mut Workspace, &Screen),
    _deactivate: fn(&dyn XConn, &mut Workspace, &Screen),
    _add_window: fn(&dyn XConn, &mut Workspace, XWindowID, &Screen),
    _del_window: fn(&dyn XConn, &mut Workspace, XWindowID, &Screen) -> Client,
}

impl Workspace {
    pub fn with_layout(layout: LayoutType) -> Self {
        todo!()
    }

    pub fn focus_window(&mut self, window: XWindowID) {
        todo!()
    }
}