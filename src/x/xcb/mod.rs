use std::convert::{TryFrom, TryInto};

use xcb_util::{ewmh, cursor};

use strum::IntoEnumIterator;

use super::{
    Atoms,
    atom::Atom,
    core::{
        XAtom,
        XWindowID, Result, 
        XError, XConn,
        StackMode,
    },
    event::{
        XEvent,
        ConfigureEvent,
        ConfigureRequestData,
        ReparentEvent,
        PropertyEvent,
        KeypressEvent,
        MouseEvent as MouseEventType,
        ClientMessageEvent,
        ClientMessageData,
    },
};
use crate::types::{
    Point, Geometry,
};
use crate::keybinds::{
    Mousebind, MouseEventKind,
};
use crate::util;

mod xconn;
mod convert;

const X_EVENT_MASK: u8 = 0x7f;

// used for casting events and stuff
macro_rules! cast {
    ($etype:ty, $event:expr) => {
        unsafe {xcb::cast_event::<$etype>(&$event)}
    };
}

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
            conn,
            root,
            atoms,
            cursor: 0,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.atoms.insert("_NET_SUPPORTED", self.conn.SUPPORTED());
        self.atoms.insert("_NET_WM_WINDOW_TYPE", self.conn.WM_WINDOW_TYPE());
        self.atoms.insert("_NET_WM_STRUT", self.conn.WM_STRUT());
        self.atoms.insert("_NET_WM_STRUT_PARTIAL", self.conn.WM_STRUT_PARTIAL());

        self.atoms.insert(
            "WM_DELETE_WINDOW",
            xcb::intern_atom(&self.conn, false, "WM_DELETE_WINDOW")
            .get_reply()?
            .atom());

        self.atoms.insert(
            "WM_TAKE_FOCUS",
            xcb::intern_atom(&self.conn, false, "WM_TAKE_FOCUS")
            .get_reply()?
            .atom());

        self.atoms.insert(
            "WM_PROTOCOLS", self.conn.WM_PROTOCOLS()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_DESKTOP", 
            self.conn.WM_WINDOW_TYPE_DESKTOP()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_DOCK",
            self.conn.WM_WINDOW_TYPE_DOCK()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_TOOLBAR",
            self.conn.WM_WINDOW_TYPE_TOOLBAR()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_MENU",
            self.conn.WM_WINDOW_TYPE_MENU()
        );
        self.atoms.insert(
            "_NET_WM_WINDOW_TYPE_UTILITY",
            self.conn.WM_WINDOW_TYPE_UTILITY()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_SPLASH",
            self.conn.WM_WINDOW_TYPE_SPLASH()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_DIALOG",
            self.conn.WM_WINDOW_TYPE_DIALOG()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_DROPDOWN_MENU",
            self.conn.WM_WINDOW_TYPE_DROPDOWN_MENU()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_NOTIFICATION",
            self.conn.WM_WINDOW_TYPE_NOTIFICATION()
        );
        self.atoms.insert(
            "_NET_WINDOW_TYPE_NORMAL",
            self.conn.WM_WINDOW_TYPE_NORMAL()
        );
        self.atoms.insert(
            "_NET_WM_STATE",
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
        Ok(xcb::change_window_attributes_checked(
            &self.conn,
            window, 
            &util::cursor_attrs(self.cursor)
        ).request_check()?)
    }

    #[allow(dead_code)]
    pub(crate) fn get_setup(&self) -> xcb::Setup<'_> {
        self.conn.get_setup()
    }

    fn process_raw_event(&self, event: xcb::GenericEvent) -> Result<XEvent> {
        use XEvent::*;

        //todo: handle randr events

        match event.response_type() & X_EVENT_MASK {
            xcb::CONFIGURE_NOTIFY => {
                let event = cast!(xcb::ConfigureNotifyEvent, event);
                if event.event() == self.root {
                    debug!("Top level window configuration")
                }
                Ok(ConfigureNotify(ConfigureEvent {
                    id: event.window(),
                    geom: Geometry {
                        x: event.x() as i32,
                        y: event.y() as i32,
                        height: event.height() as u32,
                        width: event.width() as u32
                    },
                    is_root: event.window() == self.root,
                }))
            }
            xcb::CONFIGURE_REQUEST => {
                use StackMode::*;
                use xcb::{
                    CONFIG_WINDOW_X as CF_X,
                    CONFIG_WINDOW_Y as CF_Y,
                    CONFIG_WINDOW_HEIGHT as CF_H,
                    CONFIG_WINDOW_WIDTH as CF_W,
                    CONFIG_WINDOW_STACK_MODE as CF_SM,
                    CONFIG_WINDOW_SIBLING as CF_SB,
                };

                let event = cast!(xcb::ConfigureRequestEvent, event);
                let is_root = event.window() == self.root;
                if event.parent() == self.root {
                    debug!("Top level window configuration request");
                }
                let vmask = event.value_mask();

                let parent = event.parent();
                let x = if CF_X as u16 & vmask != 0 {
                    Some(event.x() as i32)
                } else {None};
                let y = if CF_Y as u16 & vmask != 0 {
                    Some(event.y() as i32)
                } else {None};
                let height = if CF_H as u16 & vmask != 0 {
                    Some(event.height() as u32)
                } else {None};
                let width = if CF_W as u16 & vmask != 0 {
                    Some(event.width() as u32)
                } else {None};
                let stack_mode = if CF_SM as u16 & vmask != 0 {
                    match event.stack_mode() as u32 {
                        xcb::STACK_MODE_ABOVE => Some(Above),
                        xcb::STACK_MODE_BELOW => Some(Below),
                        xcb::STACK_MODE_TOP_IF => Some(TopIf),
                        xcb::STACK_MODE_BOTTOM_IF => Some(BottomIf),
                        xcb::STACK_MODE_OPPOSITE => Some(Opposite),
                        _ => None
                    }
                } else {None};
                let sibling = if CF_SB as u16 & vmask != 0 {
                    Some(event.sibling())
                } else {None};

                Ok(ConfigureRequest(ConfigureRequestData {
                    id: event.window(),
                    parent,
                    sibling,
                    x, y, height, width,
                    stack_mode,
                    is_root,
                }))
            }
            xcb::MAP_REQUEST => {
                let event = cast!(xcb::MapRequestEvent, event);

                let override_redirect = if let Ok(reply) = xcb::get_window_attributes(
                    &self.conn, event.window()
                ).get_reply() {
                    reply.override_redirect()
                } else {false};

                Ok(MapRequest(event.window(), override_redirect))
            }
            xcb::MAP_NOTIFY => {
                let event = cast!(xcb::MapNotifyEvent, event);

                Ok(MapNotify(event.window()))
            }
            xcb::UNMAP_NOTIFY => {
                let event = cast!(xcb::UnmapNotifyEvent, event);

                Ok(UnmapNotify(event.window()))
            }
            xcb::DESTROY_NOTIFY => {
                let event = cast!(xcb::DestroyNotifyEvent, event);

                Ok(DestroyNotify(event.window()))
            }
            xcb::ENTER_NOTIFY => {
                let event = cast!(xcb::EnterNotifyEvent, event);

                let grab = event.mode() as u32 == xcb::NOTIFY_MODE_GRAB;

                Ok(EnterNotify(event.event(), grab))
            }
            xcb::LEAVE_NOTIFY => {
                let event = cast!(xcb::LeaveNotifyEvent, event);

                let grab = event.mode() as u32 == xcb::NOTIFY_MODE_GRAB;

                Ok(LeaveNotify(event.event(), grab))
            }
            xcb::REPARENT_NOTIFY => {
                let event = cast!(xcb::ReparentNotifyEvent, event);

                Ok(ReparentNotify(ReparentEvent {
                    event: event.event(),
                    parent: event.parent(),
                    child: event.window(),
                    over_red: event.override_redirect(),
                }))
            }
            xcb::PROPERTY_NOTIFY => {
                let event = cast!(xcb::PropertyNotifyEvent, event);

                Ok(PropertyNotify(PropertyEvent {
                    id: event.window(),
                    atom: event.atom(),
                    time: event.time(),
                    deleted: event.state() == xcb::PROPERTY_DELETE as u8,
                }))
            }
            xcb::KEY_PRESS => {
                let event = cast!(xcb::KeyPressEvent, event);
                let numlock = xcb::MOD_MASK_2 as u16;

                Ok(KeyPress(event.event(), KeypressEvent {
                    mask: event.state() & !numlock,
                    keycode: event.detail(),
                }))
            }
            xcb::KEY_RELEASE => {
                Ok(KeyRelease)
            }
            xcb::BUTTON_PRESS => {
                let event = cast!(xcb::ButtonPressEvent, event);

                Ok(MouseEvent(MouseEventType {
                    id: event.event(),
                    location: Point {
                        x: event.root_x() as i32,
                        y: event.root_y() as i32,
                    },
                    state: Mousebind {
                        button: (event.detail() as u32).try_into()?,
                        modmask: event.state(),
                        kind: MouseEventKind::Press,
                    }
                }))
            }
            xcb::BUTTON_RELEASE => {
                let event = cast!(xcb::ButtonReleaseEvent, event);

                Ok(MouseEvent(MouseEventType {
                    id: event.event(),
                    location: Point {
                        x: event.root_x() as i32,
                        y: event.root_y() as i32,
                    },
                    state: Mousebind {
                        button: (event.detail() as u32).try_into()?,
                        modmask: event.state(),
                        kind: MouseEventKind::Release,
                    }
                }))
            }
            xcb::MOTION_NOTIFY => {
                let event = cast!(xcb::MotionNotifyEvent, event);

                Ok(MouseEvent(MouseEventType {
                    id: event.event(),
                    location: Point {
                        x: event.root_x() as i32,
                        y: event.root_y() as i32,
                    },
                    state: Mousebind {
                        button: (event.detail() as u32).try_into()?,
                        modmask: event.state(),
                        kind: MouseEventKind::Motion,
                    }
                }))
            }
            xcb::CLIENT_MESSAGE => {
                let event = cast!(xcb::ClientMessageEvent, event);

                Ok(ClientMessage(ClientMessageEvent {
                    window: event.window(),
                    data: ClientMessageData::try_from(event)?,
                    type_: event.type_(),
                }))
            }
            n => {
                Ok(Unknown(n))
            }
        }
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