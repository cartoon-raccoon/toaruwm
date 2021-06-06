use std::convert::{TryFrom, TryInto};

use xcb_util::{ewmh, cursor};

use strum::IntoEnumIterator;

use crate::x::{
    Atoms,
    core::{
        XWindowID, Result, XError, XConn,
    },
    event::ClientMessageData,
};
use crate::types::Atom as XAtom;
use crate::util;
use super::atom::Atom;

mod xconn;

/// A connection to an X server, backed by the XCB library.
/// 
/// This is a very simple connection to the X server
/// and is completely synchronous.
/// 
/// It implements [XConn][1] and thus can be used with a
/// [WindowManager][2].
/// 
/// [1]: crate::x::core::XConn
/// [2]: crate::manager::WindowManager
pub struct XCBConn {
    conn: ewmh::Connection,
    root: XWindowID,
    atoms: Atoms,
    cursor: u32,
}

impl XCBConn {
    pub fn connect() -> Result<Self> {
        let (x, idx) = xcb::Connection::connect(None)?;
        let conn = ewmh::Connection::connect(x).map_err(|(e, _)| e)?;

        
        let atoms = Atoms::new();

        let root = conn.get_setup()
            .roots()
            .nth(idx as usize)
            .expect("Could not get root id")
            .root();

        Ok(Self {
            conn: conn,
            root: root,
            atoms: atoms,
            cursor: 0,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.atoms.insert("_NET_SUPPORTED".into(), self.conn.SUPPORTED());
        self.atoms.insert("_NET_WM_WINDOW_TYPE".into(), self.conn.WM_WINDOW_TYPE());
        self.atoms.insert("_NET_WM_STRUT".into(), self.conn.WM_STRUT());
        self.atoms.insert("_NET_WM_STRUT_PARTIAL".into(), self.conn.WM_STRUT_PARTIAL());

        self.atoms.insert(
            "WM_DELETE_WINDOW".into(),
            xcb::intern_atom(&self.conn, false, "WM_DELETE_WINDOW")
            .get_reply()?
            .atom());

        self.atoms.insert(
            "WM_TAKE_FOCUS".into(),
            xcb::intern_atom(&self.conn, false, "WM_TAKE_FOCUS")
            .get_reply()?
            .atom());

        self.atoms.insert(
            "WM_PROTOCOLS".into(), self.conn.WM_PROTOCOLS()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_DESKTOP".into(), 
            self.conn.WM_WINDOW_TYPE_DESKTOP()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_DOCK".into(),
            self.conn.WM_WINDOW_TYPE_DOCK()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_TOOLBAR".into(),
            self.conn.WM_WINDOW_TYPE_TOOLBAR()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_MENU".into(),
            self.conn.WM_WINDOW_TYPE_MENU()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_UTILITY".into(),
            self.conn.WM_WINDOW_TYPE_UTILITY()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_SPLASH".into(),
            self.conn.WM_WINDOW_TYPE_SPLASH()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_DIALOG".into(),
            self.conn.WM_WINDOW_TYPE_DIALOG()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_DROPDOWN_MENU".into(),
            self.conn.WM_WINDOW_TYPE_DROPDOWN_MENU()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_NOTIFICATION".into(),
            self.conn.WM_WINDOW_TYPE_NOTIFICATION()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_NORMAL".into(),
            self.conn.WM_WINDOW_TYPE_NORMAL()
        );
        self.atoms.insert(
            "_NET_WM_STATE".into(),
            self.conn.WM_STATE()
        );

        for atom in Atom::iter() {
            self.atoms.insert(atom.as_ref(), self.atom(atom.as_ref())?)
        }

        self.create_cursor(cursor::LEFT_PTR)?;
        self.set_cursor(self.root)?;

        //todo: randr setup

        Ok(())
    }

    pub fn add_atom<S: AsRef<str>>(&mut self, name: S, atom: XAtom) {
        self.atoms.insert(name.as_ref(), atom);
    }

    pub fn atoms(&self) -> &Atoms {
        &self.atoms
    }

    pub fn conn(&self) -> &xcb::Connection {
        &self.conn
    }

    //todo: make this better
    pub fn create_cursor(&mut self, glyph: u16) -> Result<()> {
        debug!("Creating cursor");
        let cursor_id = cursor::create_font_cursor_checked(&self.conn, glyph)?;
        self.cursor = cursor_id;
        Ok(())
    }

    pub fn set_cursor(&self, window: XWindowID) -> Result<()> {
        debug!("Setting cursor for {}", window);
        self.change_window_attributes(window, &util::cursor_attrs(self.cursor))
    }

    #[allow(dead_code)]
    pub(crate) fn get_setup(&self) -> xcb::Setup<'_> {
        self.conn.get_setup()
    }
}

use std::string::FromUtf8Error;

impl From<FromUtf8Error> for XError {
    fn from(e: FromUtf8Error) -> XError {
        XError::InvalidPropertyData(
            format!("Invalid UTF8 data: {}", e)
        )
    }
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
            _ => {Err(XError::ConversionError)}
        }
    }
}