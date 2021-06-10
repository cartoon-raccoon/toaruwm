use std::convert::{TryFrom, TryInto};
use std::cell::Cell;

use xcb_util::{ewmh, cursor};
use xcb::randr;

use strum::IntoEnumIterator;

use super::{
    Atoms,
    atom::Atom,
    core::{
        XAtom, XWindow,
        XWindowID, Result, 
        XError, XConn,
        StackMode,
        WindowClass,
    },
    event::{
        XEvent,
        ConfigureEvent,
        ConfigureRequestData,
        ReparentEvent,
        PropertyEvent,
        KeypressEvent,
        PointerEvent,
        MouseEvent as MouseEventType,
        ClientMessageEvent,
        ClientMessageData,
    },
    property::{
        Property,
        WmHints,
        WmSizeHints,
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

#[cfg(test)]
mod tests;

const X_EVENT_MASK: u8 = 0x7f;

const MAX_LONG_LENGTH: u32 = 1024;

const RANDR_MAJ: u32 = 1;
const RANDR_MIN: u32 = 4;

// used for casting events and stuff
macro_rules! cast {
    ($etype:ty, $event:expr) => {
        unsafe {xcb::cast_event::<$etype>(&$event)}
    };
}

/// A connection to an X server, backed by the XCB library.
/// 
/// This is a very simple connection to the X server
/// and is completely synchronous, despite the async capabilities
/// of the underlying xcb library.
/// 
/// It implements [XConn][1] and thus can be used with a
/// [WindowManager][2].
/// 
/// [1]: crate::x::core::XConn
/// [2]: crate::manager::WindowManager
pub struct XCBConn {
    conn: ewmh::Connection,
    root: XWindow,
    idx: i32,
    randr_base: u8,
    atoms: Cell<Atoms>,
    cursor: u32,
}

impl XCBConn {
    /// Connect to the X server and allocate a new Connection.
    /// 
    /// This also initialises the randr extension.
    pub fn connect() -> Result<Self> {
        // initialize xcb connection
        let (x, idx) = xcb::Connection::connect(None)?;
        debug!("Connected to x server, got preferred screen {}", idx);
        // wrap it in an ewmh connection just for fun
        let conn = ewmh::Connection::connect(x).map_err(|(e, _)| e)?;

        // initialize our atom handler
        let atoms = Cell::new(Atoms::new());

        Ok(Self {
            conn,
            root: XWindow::zeroed(),
            idx,
            randr_base: 0,
            atoms,
            cursor: 0,
        })
    }

    pub fn init(&mut self) -> Result<()> {

        // validate randr version
        let res = randr::query_version(&self.conn, RANDR_MAJ, RANDR_MIN)
            .get_reply()?;

        let (maj, min) = (res.major_version(), res.minor_version());

        debug!("Got randr version {}.{}", maj, min);

        if maj != RANDR_MAJ || min < RANDR_MIN {
            return Err(XError::RandrError(
                format!(
                    "Received randr version {}.{}, requires v{}.{} or higher",
                    maj, min, RANDR_MAJ, RANDR_MIN
                )
            ))
        }

        // get root window id
        self.root = match self.conn.get_setup().roots().nth(self.idx as usize) {
            Some(root) => {
                let geom = self.get_geometry(root.root())?;
                XWindow::with_data(root.root(), geom)
            },
            None => return Err(XError::NoScreens),
        };
        debug!("Got root: {:?}", self.root);

        // initialize randr and get its event mask
        self.randr_base = self.conn.get_extension_data(&mut randr::id())
            .ok_or_else(|| XError::RandrError("could not load randr".into()))?
            .first_event();

        debug!("Got randr_base {}", self.randr_base);

        
        // intern all known atoms
        for atom in Atom::iter() {
            let atom_val = self.atom(atom.as_ref())?;
            self.atoms.get_mut().insert(atom.as_ref(), atom_val);
        }

        // initialize cursor and set it for the root screen
        self.create_cursor(cursor::LEFT_PTR)?;
        self.set_cursor(self.root.id)?;

        Ok(())
    }

    pub fn add_atom<S: AsRef<str>>(&mut self, name: S, atom: XAtom) {
        self.atoms.get_mut().insert(name.as_ref(), atom);
    }

    pub fn atoms(&self) -> &Atoms {
        // SAFETY: returns an immutable reference
        unsafe {&*self.atoms.as_ptr()}
    }

    pub fn conn(&self) -> &xcb::Connection {
        &self.conn
    }

    #[cfg(test)]
    pub fn ewmh_conn(&self) -> &ewmh::Connection {
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

    pub(crate) fn get_setup(&self) -> xcb::Setup<'_> {
        self.conn.get_setup()
    }

    pub(crate) fn check_win(&self) -> Result<XWindowID> {
        self.create_window(
            WindowClass::CheckWin, 
            Geometry::new(0, 0, 1, 1,),
            false,
        )
    }

    pub(crate) fn screen(&self, idx: usize) -> Result<xcb::Screen<'_>> {
        let mut roots: Vec<_> = self.get_setup().roots().collect();

        if idx >= roots.len() {
            Err(XError::InvalidScreen)
        } else {
            Ok(roots.remove(idx))
        }
    }

    pub(crate) fn depth<'a>(&self, screen: &'a xcb::Screen<'_>) -> Result<xcb::Depth<'a>> {
        screen.allowed_depths()
            .max_by(|x, y| x.depth().cmp(&y.depth()))
            .ok_or(XError::RequestError("get depth"))
    }

    pub(crate) fn visual_type(&self, depth: &xcb::Depth<'_>) -> Result<xcb::Visualtype> {
        depth.visuals()
            .find(|v| v.class() == xcb::VISUAL_CLASS_TRUE_COLOR as u8)
            .ok_or(XError::RequestError("get visual type"))
    }

    fn process_raw_event(&self, event: xcb::GenericEvent) -> Result<XEvent> {
        use XEvent::*;

        let etype = event.response_type() & X_EVENT_MASK;

        if etype == self.randr_base + randr::NOTIFY {
            return Ok(XEvent::RandrNotify)
        } else if etype == self.randr_base + randr::SCREEN_CHANGE_NOTIFY {
            return Ok(XEvent::ScreenChange)
        }

        match etype {
            xcb::CONFIGURE_NOTIFY => {
                let event = cast!(xcb::ConfigureNotifyEvent, event);
                if event.event() == self.root.id {
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
                    is_root: event.window() == self.root.id,
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
                let is_root = event.window() == self.root.id;
                if event.parent() == self.root.id {
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

                let id = event.event();
                let abs = Point::new(event.root_x() as i32, event.root_y() as i32);
                let rel = Point::new(event.event_x() as i32, event.event_y() as i32);

                let ptrev = PointerEvent {id, abs, rel};

                Ok(EnterNotify(ptrev, grab))
            }
            xcb::LEAVE_NOTIFY => {
                let event = cast!(xcb::LeaveNotifyEvent, event);

                let grab = event.mode() as u32 == xcb::NOTIFY_MODE_GRAB;

                let id = event.event();
                let abs = Point::new(event.root_x() as i32, event.root_y() as i32);
                let rel = Point::new(event.event_x() as i32, event.event_y() as i32);

                let ptrev = PointerEvent {id, abs, rel};

                Ok(LeaveNotify(ptrev, grab))
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

    fn get_prop_atom(&self, prop: XAtom, window: XWindowID) -> Result<Option<Property>> {
        let r = xcb::get_property(
            &self.conn,
            false,
            window,
            prop,
            xcb::ATOM_ANY,
            // start at offset 0
            0, 
            // allow for up to 4 * MAX_LONG_LENGTH bytes of information
            MAX_LONG_LENGTH,
        ).get_reply()?;

        if r.type_() == xcb::NONE {
            debug!("prop type is none");
            return Ok(None)
        }

        let prop_type = self.lookup_atom(r.type_())?;
        debug!("got prop_type {}", prop_type);

        Ok(match prop_type.as_str() {
            "ATOM" => Some(Property::Atom(
                r.value()
                    .iter()
                    .map(|a| self.lookup_atom(*a).unwrap_or_else(|_| "".into()))
                    .collect::<Vec<String>>()
            )),
            "CARDINAL" => Some(Property::Cardinal(r.value()[0])),
            "STRING" => Some(Property::String(
                String::from_utf8_lossy(&r.value().to_vec())
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect()
            )),
            "UTF8_STRING" => Some(Property::UTF8String(
                String::from_utf8(r.value().to_vec())?
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect()
            )),
            "WINDOW" => Some(Property::Window(r.value().to_vec())),
            "WM_HINTS" => Some(Property::WMHints(
                WmHints::try_from_bytes(r.value())?
            )),
            "WM_SIZE_HINTS" => Some(Property::WMSizeHints(
                WmSizeHints::try_from_bytes(r.value())?
            )),
            n => {
                if n == "WM_STATE" {
                    debug!("Type is WM_STATE");
                }
                match r.format() {
                    8 => Some(Property::U8List(
                        n.into(),
                        r.value::<u8>().into()
                    )),
                    16 => Some(Property::U16List(
                        n.into(),
                        r.value::<u16>().into()
                    )),
                    32 => Some(Property::U32List(
                        n.into(),
                        r.value::<u32>().into()
                    )),
                    n => {
                        return Err(
                            XError::InvalidPropertyData(
                                format!("received format {}", n)
                            )
                        )
                    },
                }
            }
        })
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