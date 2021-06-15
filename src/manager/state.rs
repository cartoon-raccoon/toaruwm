use super::{WindowManager};

use crate::core::{Ring, Workspace, Desktop};
use crate::x::{XConn, XWindow, XWindowID};

/// Provides a view into the state of the window manager.
///
/// Used as a context when generating event actions.
pub(crate) struct WMState<'a, X: XConn> {
    pub conn: &'a X,
    pub workspaces: &'a Ring<Workspace>,
    pub desktop: &'a Desktop,
    pub root: XWindow,
    pub selected: Option<XWindowID>,
    pub focused: Option<XWindowID>,
}

impl<X: XConn> WindowManager<X> {
    pub(crate) fn state(&self) -> WMState<X> {
        WMState {
            conn: &self.conn,
            workspaces: &self.desktop.workspaces,
            desktop: &self.desktop,
            root: self.root,
            selected: self.selected,
            focused: self.focused,
        }
    }
}