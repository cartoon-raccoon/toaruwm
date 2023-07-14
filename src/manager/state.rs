use super::WindowManager;

use custom_debug_derive::Debug;

use crate::core::{Client, Desktop, Ring, Workspace};
use crate::x::{XConn, XWindow, XWindowID};

/// The state that the current window manager is in.
#[non_exhaustive]
#[derive(std::fmt::Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum State {

}

/// Provides introspection into the state of the window manager.
///
/// The `'wm` lifetime refers to the lifetime of the parent
/// `WindowManager` type.
#[derive(Debug, Clone, Copy)]
pub struct WmState<'wm, X: XConn> {
    /// The `XConn` implementation currently being used.
    #[debug(skip)]
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
    pub fn state(&self) -> WmState<'_, X> {
        WmState {
            conn: &self.conn,
            workspaces: &self.desktop.workspaces,
            desktop: &self.desktop,
            root: self.root,
            selected: self.selected,
            focused: self.focused,
        }
    }
}

impl<'wm, X: XConn> WmState<'wm, X> {
    /// Looks up a client with the given X ID.
    pub fn lookup_client(&self, id: XWindowID) -> Option<&Client> {
        self.desktop.current().windows.lookup(id)
    }
}
