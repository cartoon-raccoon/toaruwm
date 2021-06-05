use std::collections::HashSet;

use xcb_util::ewmh;

use crate::x::core::{XWindowID, Result, XError};
use crate::core::types::Atom;

#[allow(dead_code)]
pub struct XCBConn {
    conn: ewmh::Connection,
    root: XWindowID,
    atoms: HashSet<Atom>,
}

impl From<xcb::ConnError> for XError {
    fn from(_: xcb::ConnError) -> XError {
        XError::Connection
    }
}

impl From<xcb::GenericError> for XError {
    fn from(from: xcb::GenericError) -> XError {
        XError::ServerError(from.to_string())
    }
}

impl XCBConn {
    pub fn connect() -> Result<Self> {
        let (x, idx) = xcb::Connection::connect(None)?;
        let conn = ewmh::Connection::connect(x).map_err(|(e, _)| e)?;

        let root = conn.get_setup()
            .roots()
            .nth(idx as usize)
            .expect("Could not get root id")
            .root();
        
        let mut atoms = HashSet::new();

        atoms.insert(conn.SUPPORTED());

        atoms.insert(xcb::intern_atom(&conn, false, "WM_DELETE_WINDOW")
            .get_reply()?
            .atom());

        atoms.insert(xcb::intern_atom(&conn, false, "WM_TAKE_FOCUS")
            .get_reply()?
            .atom());

        atoms.insert(conn.WM_PROTOCOLS());
        atoms.insert(conn.WM_WINDOW_TYPE_DESKTOP());
        atoms.insert(conn.WM_WINDOW_TYPE_DOCK());
        atoms.insert(conn.WM_WINDOW_TYPE_TOOLBAR());
        atoms.insert(conn.WM_WINDOW_TYPE_MENU());
        atoms.insert(conn.WM_WINDOW_TYPE_UTILITY());
        atoms.insert(conn.WM_WINDOW_TYPE_SPLASH());
        atoms.insert(conn.WM_WINDOW_TYPE_DIALOG());
        atoms.insert(conn.WM_WINDOW_TYPE_DROPDOWN_MENU());
        atoms.insert(conn.WM_WINDOW_TYPE_NOTIFICATION());
        atoms.insert(conn.WM_WINDOW_TYPE_NORMAL());
        atoms.insert(conn.WM_STATE());

        //todo: insert interned atoms

        Ok(Self {
            conn: conn,
            root: root,
            atoms: atoms,
        })
    }
}