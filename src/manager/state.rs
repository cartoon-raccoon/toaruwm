use super::{WindowManager, MouseMode};

use crate::core::{Ring, Workspace};
use crate::x::{XConn, XWindowID};

/// Provides a view into the state of the window manager.
///
/// Used as a context when generating event actions.
pub(crate) struct WMState<'a, X: XConn> {
    pub conn: &'a X,
    pub workspaces: &'a Ring<Workspace>,
    pub root: u32,
    pub mousemode: MouseMode,
    pub selected: Option<XWindowID>,
}

impl<X: XConn> WindowManager<X> {
    pub(crate) fn state(&self) -> WMState<X> {
        WMState {
            conn: &self.conn,
            workspaces: &self.desktop.workspaces,
            root: self.root,
            mousemode: self.mousemode,
            selected: self.selected,
        }
    }
}