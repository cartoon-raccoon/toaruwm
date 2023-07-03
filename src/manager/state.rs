use super::{WindowManager};

use crate::core::{Ring, Workspace, Desktop, Client};
use crate::x::{XConn, XWindow, XWindowID};

/// Provides a view into the state of the window manager.
/// It is used as a context when generating event actions.
/// 
/// The `'wm` lifetime refers to the lifetime of the parent
/// `WindowManager` type.
pub(crate) struct WMState<'wm, X: XConn> {
    pub conn: &'wm X,
    pub workspaces: &'wm Ring<Workspace>,
    pub desktop: &'wm Desktop,
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

impl <'wm, X: XConn> WMState<'wm, X> {
    pub fn lookup_client(&self, id: XWindowID) -> Option<&Client> {
        self.desktop.current().windows.lookup(id)
    }
}