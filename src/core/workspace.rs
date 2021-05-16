use crate::core::{
    window::{Client, ClientRing},
    desktop::Screen,
};
use crate::layouts::LayoutType;
use crate::x::{XConn, XWindowID};

pub struct Workspace {
    pub(crate) windows: ClientRing,
    pub(crate) master: Option<XWindowID>,
    pub(crate) layout: LayoutType,

    _activate: fn(&dyn XConn, &mut Workspace, &Screen),
    _deactivate: fn(&dyn XConn, &mut Workspace, &Screen),
    _add_window: fn(&dyn XConn, &mut Workspace, &Screen),
}