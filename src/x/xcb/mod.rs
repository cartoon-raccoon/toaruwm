use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use xcb_util::ewmh;

use crate::x::{
    core::{
        XWindowID, Result, XError,
    },
    event::ClientMessageData,
};
use crate::types::Atom;

mod impl_xconn;

pub struct XCBConn {
    conn: ewmh::Connection,
    root: XWindowID,
    atoms: HashMap<String, Atom>,
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
        
        let mut atoms = HashMap::new();

        atoms.insert("_NET_SUPPORTED".into(), conn.SUPPORTED());
        atoms.insert("_NET_WM_WINDOW_TYPE".into(), conn.WM_WINDOW_TYPE());
        atoms.insert("_NET_WM_STRUT".into(), conn.WM_STRUT());
        atoms.insert("_NET_WM_STRUT_PARTIAL".into(), conn.WM_STRUT_PARTIAL());

        atoms.insert(
            "WM_DELETE_WINDOW".into(),
            xcb::intern_atom(&conn, false, "WM_DELETE_WINDOW")
            .get_reply()?
            .atom());

        atoms.insert(
            "WM_TAKE_FOCUS".into(),
            xcb::intern_atom(&conn, false, "WM_TAKE_FOCUS")
            .get_reply()?
            .atom());

        atoms.insert(
            "WM_PROTOCOLS".into(), conn.WM_PROTOCOLS()
        );
        atoms.insert(
            "_NET_WM_WINDOW_TYPE_DESKTOP".into(), 
            conn.WM_WINDOW_TYPE_DESKTOP()
        );
        atoms.insert(
            "_NET_WM_WINDOW_TYPE_DOCK".into(),
            conn.WM_WINDOW_TYPE_DOCK()
        );
        atoms.insert(
            "_NET_WM_WINDOW_TYPE_TOOLBAR".into(),
            conn.WM_WINDOW_TYPE_TOOLBAR()
        );
        atoms.insert(
            "_NET_WM_WINDOW_TYPE_MENU".into(),
            conn.WM_WINDOW_TYPE_MENU()
        );
        atoms.insert(
            "_NET_WM_WINDOW_TYPE_UTILITY".into(),
            conn.WM_WINDOW_TYPE_UTILITY()
        );
        atoms.insert(
            "_NET_WINDOW_TYPE_SPLASH".into(),
            conn.WM_WINDOW_TYPE_SPLASH()
        );
        atoms.insert(
            "_NET_WINDOW_TYPE_DIALOG".into(),
            conn.WM_WINDOW_TYPE_DIALOG()
        );
        atoms.insert(
            "_NET_WINDOW_TYPE_DROPDOWN_MENU".into(),
            conn.WM_WINDOW_TYPE_DROPDOWN_MENU()
        );
        atoms.insert(
            "_NET_WINDOW_TYPE_NOTIFICATION".into(),
            conn.WM_WINDOW_TYPE_NOTIFICATION()
        );
        atoms.insert(
            "_NET_WINDOW_TYPE_NORMAL".into(),
            conn.WM_WINDOW_TYPE_NORMAL()
        );
        atoms.insert(
            "_NET_WM_STATE".into(),
            conn.WM_STATE()
        );

        Ok(Self {
            conn: conn,
            root: root,
            atoms: atoms,
        })
    }

    pub fn add_atom<S: Into<String>>(&mut self, name: S, atom: Atom) {
        self.atoms.insert(name.into(), atom);
    }
}

use std::array::TryFromSliceError as TFSError;

impl From<TFSError> for XError {
    fn from(_: TFSError) -> Self {
        XError::ConversionError
    }
}

impl TryFrom<&xcb::xproto::ClientMessageEvent> for ClientMessageData {
    type Error = XError;

    fn try_from(event: &xcb::xproto::ClientMessageEvent) -> Result<Self> {
        let data = event.data();
        match event.format() {
            8 => {
                Ok(Self::U8(data.data8()[0..20]
                .try_into()?))
            }
            16 => {
                Ok(Self::U16(data.data16()[0..10]
                .try_into()?))
            }
            32 => {
                Ok(Self::U32(data.data32()[0..5]
                .try_into()?))
            }
            _ => {unreachable!()}
        }
    }
}

// pub fn from_event(event: &xproto::ClientMessageEvent) -> Self {
    //     let data = event.data();
    //     match event.format() {
    //         8 => {
    //             Self::U8(data.data8()[0..20]
    //             .try_into().expect("Byte: Incorrect conversion"))
    //         }
    //         16 => {
    //             Self::U16(data.data16()[0..10]
    //             .try_into().expect("Word: Incorrect conversion"))
    //         }
    //         32 => {
    //             Self::U32(data.data32()[0..5]
    //             .try_into().expect("DWord: Incorrect conversion"))
    //         }
    //         _ => {unreachable!()}
    //     }
    // }