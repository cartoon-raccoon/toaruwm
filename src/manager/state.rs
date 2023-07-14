use super::WindowManager;

use crate::core::{Client, Desktop, Ring, Workspace};
use crate::x::{XConn, XWindow, XWindowID};

/// The state that the current window manager is in.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum State {

}

/// Provides introspection into the state of the window manager.
/// It is used as a context when generating event actions.
///
/// The `'wm` lifetime refers to the lifetime of the parent
/// `WindowManager` type.
#[derive(Clone)]
pub struct WMState<'wm, X: XConn> {
    /// The `XConn` implementation currently being used.
    pub conn: &'wm X,
    /// The workspaces maintained by the window manager.
    pub workspaces: &'wm Ring<Workspace>,
    /// The root window.
    pub root: XWindow,
    /// The selected window, if any.
    pub selected: Option<XWindowID>,
    /// The currently focused window, if any.
    pub focused: Option<XWindowID>,
    pub(crate) desktop: &'wm Desktop,
}

//todo: implement debug!

impl<X: XConn> WindowManager<X> {
    /// Provides a WMState for introspection.
    pub fn state(&self) -> WMState<'_, X> {
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

impl<'wm, X: XConn> WMState<'wm, X> {
    /// Looks up a client with the given X ID.
    pub fn lookup_client(&self, id: XWindowID) -> Option<&Client> {
        self.desktop.current().windows.lookup(id)
    }
}
