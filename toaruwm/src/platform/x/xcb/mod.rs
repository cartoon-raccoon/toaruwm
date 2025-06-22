//! Implementation of `XConn` backed by the `xcb` library.
//!
//! This module provides an interface to the X11 protocol via the XCB
//! backend.
//!
//! The core of this module is `XCBConn`, a type that implements the
//! `XConn` trait and can thus serve as a Connection within a
//! `WindowManager`.
//!
//! NOTE: As of `xcb` 1.2.1, there is a bug in the library that causes
//! panics due to a misaligned pointer dereference. You should use
//! `x11rb` instead.
use core::marker::PhantomData;

use std::cell::{Cell, RefCell};
use std::fmt;

use tracing::debug;

use xcb::{randr, xkb};
use xcb::x;
use xcb::{Xid as XCBid, XidNew};

use strum::*;

use super::{
    atom::Atom,
    core::{
        Result, StackMode, WindowClass, XAtom, XCore, XError, XWindow, XWindowID, Xid,
        RandR, Xkb, RandrErrorKind, XKBErrorKind,
    },
    cursor,
    event::{
        ClientMessageData, ClientMessageEvent, ConfigureEvent, ConfigureRequestData, KeypressEvent,
        PointerEvent, PropertyEvent, ReparentEvent, XEvent,
    },
    property::{Property, WmHints, WmSizeHints},
    Atoms, ConnStatus, Initialized, Uninitialized,
};
use crate::bindings::ButtonIndex;
use crate::types::{Rectangle, Point, Logical};

mod convert;
mod util;
mod xconn;

use util::{cast, id, req_and_check, req_and_reply};

use super::{
    MAX_LONG_LENGTH,
    RANDR_MAJ, RANDR_MIN,
    XKB_MAJ, XKB_MIN,
};

/// A connection to an X server, backed by the XCB library.
///
/// This is a very simple connection to the X server
/// and is completely synchronous, despite the async capabilities
/// of the underlying xcb library.
///
/// It implements [XConn][1] and thus can be used with a
/// [WindowManager][2].
///
/// # Usage
///
/// ```no_run
/// use toaruwm::x::xcb::XCBConn;
///
/// let conn = XCBConn::connect().expect("Connection error");
/// let mut conn = conn.init().expect("Could not initialize");
///
/// /* or: */
/// let mut conn = XCBConn::new().expect("Connection error");
/// ```
///
/// [1]: crate::x::core::XConn
/// [2]: crate::manager::WindowManager
pub struct XCBConn<S: ConnStatus> {
    conn: xcb::Connection,
    root: XWindow,
    idx: i32,
    randr_base: u8,
    atoms: RefCell<Atoms>, // wrap in RefCell for interior mutability
    cursor: x::Cursor,
    mousemode: Cell<Option<ButtonIndex>>, // ditto
    _marker: PhantomData<S>,
}

impl XCBConn<Uninitialized> {
    /// Connect to the X server and allocate a new Connection.
    pub fn connect() -> Result<Self> {
        // initialize xcb connection
        let (conn, idx) = xcb::Connection::connect(None)?;
        debug!("Connected to x server, got preferred screen {}", idx);
        // wrap it in an ewmh connection just for fun

        // initialize our atom handler
        let atoms = RefCell::new(Atoms::new());
        let cursor = conn.generate_id();

        Ok(Self {
            conn,
            root: XWindow::zeroed(),
            idx,
            randr_base: 0,
            atoms,
            cursor,
            mousemode: Cell::new(None),
            _marker: PhantomData,
        })
    }

    /// Initializes the connection.
    ///
    /// It does the following:
    ///
    /// - Verifies the randr version is compatible.
    /// - Initializes the RandR and XKB extensions.
    /// - Initializes the root window and its dimensions.
    /// - Interns all known [atoms][1].
    /// - Creates and sets the cursor.
    ///
    /// [1]: crate::x::Atom
    #[must_use = 
        "this consumes the connection and returns an initialized one"]
    pub fn init(mut self) -> Result<XCBConn<Initialized>> {

        // initialize randr and validate version
        let randr_base = self.initialize_randr()?;

        // initialize xkb
        self.initialize_xkb()?;

        // get root window id
        let root = match self.conn.get_setup().roots().nth(self.idx as usize) {
            Some(screen) => {
                let id = id!(screen.root());
                let geom = self.get_geometry_inner(id)?;
                XWindow::with_data(id, geom)
            }
            None => return Err(XError::NoScreens),
        };
        debug!("Got root: {:?}", self.root);

        let atomcount = Atom::iter().count();
        let mut atomvec = Vec::with_capacity(atomcount);

        // intern all known atoms

        // get cookies for all first
        for atom in Atom::iter() {
            atomvec.push((
                atom.to_string(),
                self.conn.send_request(&x::InternAtom {
                    only_if_exists: false,
                    name: atom.as_ref().as_bytes(),
                }),
            ));
        }

        let atoms = self.atoms.get_mut();

        // then get replies
        for (name, cookie) in atomvec {
            atoms.insert(&name, id!(self.conn.wait_for_reply(cookie)?.atom()));
        }

        // initialize cursor and set it for the root screen
        let cursor = self.create_cursor_inner(cursor::LEFT_PTR)?;
        self.set_cursor_inner(root.id, cursor)?;

        Ok(XCBConn {
            conn: self.conn,
            root,
            idx: self.idx,
            randr_base,
            atoms: self.atoms,
            cursor,
            mousemode: self.mousemode,
            _marker: PhantomData,
        })
    }
}

impl<S: ConnStatus> XCBConn<S> {
    #[inline]
    pub(crate) fn get_geometry_inner(&self, window: XWindowID) -> Result<Rectangle<Logical>> {
        trace!("Getting geometry for window {}", window);

        // send the request and grab its reply
        Ok(req_and_reply!(
            self.conn,
            &x::GetGeometry {
                drawable: x::Drawable::Window(cast!(x::Window, *window))
            }
        )
        .map(|ok| Rectangle::new(
            ok.x() as i32,
            ok.y() as i32,
            ok.height() as i32,
            ok.width() as i32,
        ))?)
    }

    #[inline]
    pub(crate) fn create_cursor_inner(&self, glyph: u16) -> Result<x::Cursor> {
        trace!("creating cursor");

        let fid: x::Font = self.conn.generate_id();
        req_and_check!(
            self.conn,
            &x::OpenFont {
                fid,
                name: "cursor".as_bytes()
            }
        )?;

        let cid: x::Cursor = self.conn.generate_id();
        req_and_check!(
            self.conn,
            &x::CreateGlyphCursor {
                cid,
                source_font: fid,
                mask_font: fid,
                source_char: glyph,
                mask_char: glyph + 1,
                fore_red: 0,
                fore_green: 0,
                fore_blue: 0,
                back_red: 0xffff,
                back_blue: 0xffff,
                back_green: 0xffff,
            }
        )?;

        Ok(cid)
    }

    #[inline]
    pub(crate) fn set_cursor_inner(&self, window: XWindowID, cursor: x::Cursor) -> Result<()> {
        trace!("setting cursor for {}", window);

        req_and_check!(
            self.conn,
            &x::ChangeWindowAttributes {
                window: cast!(x::Window, *window),
                value_list: &[x::Cw::Cursor(cursor)]
            }
        )?;

        Ok(())
    }
}

impl<S: ConnStatus> RandR for XCBConn<S> {
    fn initialize_randr(&self) -> Result<u8> {
        let reply = req_and_reply!(
            self.conn,
            &x::QueryExtension {
                name: randr::XNAME.as_bytes()
            }
        )?;

        if !reply.present() {
            return Err(XError::RandrError(RandrErrorKind::NotPresent))
        }

        let res = req_and_reply!(
            self.conn,
            &randr::QueryVersion {
                major_version: RANDR_MAJ,
                minor_version: RANDR_MIN
            }
        )?;

        let (maj, min) = (res.major_version(), res.minor_version());

        debug!("Got randr version {}.{}", maj, min);

        if maj != RANDR_MAJ || min < RANDR_MIN {
            return Err(XError::RandrError(RandrErrorKind::IncompatibleVer(maj, min)));
        }

        // get randr event mask
        let randr_base = randr::get_extension_data(&self.conn)
            .ok_or_else(|| XError::RandrError(
                RandrErrorKind::Other("could not load randr".into())))?
            .first_event;

        trace!("Got randr_base {}", randr_base);

        Ok(randr_base)
    }
}

impl<S: ConnStatus> Xkb for XCBConn<S> {
    fn initialize_xkb(&self) -> Result<()> {
        let reply = req_and_reply!(
            self.conn,
            &x::QueryExtension {
                name: xkb::XNAME.as_bytes()
            }
        )?;

        if !reply.present() {
            return Err(XError::XKBError(XKBErrorKind::NotPresent))
        }

        let reply = req_and_reply!(
            self.conn,
            &xkb::UseExtension {
                wanted_major: XKB_MAJ,
                wanted_minor: XKB_MIN,
            }
        )?;

        let (maj, min) = (reply.server_major(), reply.server_minor());

        debug!("Got XKB version {}.{}", maj, min);

        if !(maj == XKB_MAJ || min == XKB_MIN) {
            trace!("re-initializing XKB to match versions");
            req_and_reply!(
                self.conn,
                &xkb::UseExtension {
                    wanted_major: maj,
                    wanted_minor: min,
                }
            )?;
        }

        Ok(())
    }
}

impl XCBConn<Initialized> {
    /// Shortcut static method to directly connect and
    /// initialize a new connection.
    pub fn new() -> Result<Self> {
        XCBConn::connect()?.init()
    }

    /// Adds an atom to internal atom storage.
    pub fn add_atom<N: AsRef<str>>(&mut self, name: N, atom: XAtom) {
        self.atoms.get_mut().insert(name.as_ref(), atom);
    }

    /// Returns a reference to its internal atom storage.
    pub fn atoms(&self) -> &Atoms {
        // SAFETY: returns an immutable reference
        unsafe { &*self.atoms.as_ptr() }
    }

    /// Exposes `XCBConn`'s internal connection.
    pub fn conn(&self) -> &xcb::Connection {
        &self.conn
    }

    /// Allocates a new cursor on the X server.
    pub fn create_cursor(&mut self, glyph: u16) -> Result<()> {
        self.cursor = self.create_cursor_inner(glyph)?;
        Ok(())
    }

    /// Sets the cursor for the given window.
    pub fn set_cursor(&self, window: XWindowID) -> Result<()> {
        self.set_cursor_inner(window, self.cursor)
    }

    pub(crate) fn check_win(&self) -> Result<XWindowID> {
        self.create_window(WindowClass::CheckWin, Rectangle::new(0, 0, 1, 1), false)
    }

    pub(crate) fn screen(&self, idx: usize) -> Result<&x::Screen> {
        let mut roots: Vec<_> = self.conn.get_setup().roots().collect();

        if idx >= roots.len() {
            Err(XError::InvalidScreen)
        } else {
            Ok(roots.remove(idx))
        }
    }

    pub(crate) fn depth<'c>(&self, screen: &'c x::Screen) -> Result<&'c x::Depth> {
        screen
            .allowed_depths()
            .max_by(|x, y| x.depth().cmp(&y.depth()))
            .ok_or(XError::RequestError("get depth"))
    }

    pub(crate) fn visual_type<'c>(&self, depth: &'c x::Depth) -> Result<&'c x::Visualtype> {
        depth
            .visuals()
            .iter()
            .find(|v| v.class() == x::VisualClass::TrueColor)
            .ok_or(XError::RequestError("get visual type"))
    }

    fn process_raw_event(&self, event: xcb::Event) -> Result<XEvent> {
        use randr::Event as REvent;
        use xcb::Event;

        match event {
            Event::X(event) => self.process_x_event(event),
            Event::RandR(event) => {
                match event {
                    //todo: account for the unused enum values
                    REvent::Notify(_) => Ok(XEvent::RandrNotify),
                    REvent::ScreenChangeNotify(_) => Ok(XEvent::ScreenChange),
                }
            }
            unk => Ok(XEvent::Unknown(format!("{:?}", unk))),
        }
    }

    //#[instrument(target = "xcbconn", level = "trace", skip(self))]
    fn process_x_event(&self, event: x::Event) -> Result<XEvent> {
        use x::Event;
        match event {
            Event::ConfigureNotify(event) => Ok(XEvent::ConfigureNotify(ConfigureEvent {
                from_root: id!(event.event()) == self.root.id,
                id: id!(event.window()),
                geom: Rectangle::new(
                    event.x() as i32,
                    event.y() as i32,
                    event.height() as i32,
                    event.width() as i32,
                ),
                is_root: id!(event.window()) == self.root.id,
            })),
            Event::ConfigureRequest(req) => {
                use x::{ConfigWindowMask as CWMask, StackMode as XStackMode};
                use StackMode::*;

                // extract window ids
                let id = id!(req.window());
                let parent = id!(req.parent());
                let is_root = id == self.root.id;
                if id == self.root.id {
                    trace!("Top level window configuration request");
                }

                // extract relevant values using the value mask
                let vmask = req.value_mask();
                let x = if vmask.contains(CWMask::X) {
                    Some(req.x() as i32)
                } else {
                    None
                };
                let y = if vmask.contains(CWMask::Y) {
                    Some(req.y() as i32)
                } else {
                    None
                };
                let height = if vmask.contains(CWMask::HEIGHT) {
                    Some(req.height() as u32)
                } else {
                    None
                };
                let width = if vmask.contains(CWMask::WIDTH) {
                    Some(req.width() as u32)
                } else {
                    None
                };
                let stack_mode = if vmask.contains(CWMask::STACK_MODE) {
                    let sib = if req.sibling() != x::WINDOW_NONE {
                        Some(id!(req.sibling()))
                    } else {
                        None
                    };
                    match req.stack_mode() {
                        XStackMode::Above => Some(Above(sib)),
                        XStackMode::Below => Some(Below(sib)),
                        XStackMode::TopIf => Some(TopIf(sib)),
                        XStackMode::BottomIf => Some(BottomIf(sib)),
                        XStackMode::Opposite => Some(Opposite(sib)),
                    }
                } else {
                    None
                };
                let sibling = if vmask.contains(CWMask::SIBLING) {
                    Some(id!(req.sibling()))
                } else {
                    None
                };

                Ok(XEvent::ConfigureRequest(ConfigureRequestData {
                    id,
                    parent,
                    sibling,
                    x,
                    y,
                    height,
                    width,
                    stack_mode,
                    is_root,
                }))
            }
            Event::MapRequest(req) => {
                let override_redirect = req_and_reply!(
                    self.conn,
                    &x::GetWindowAttributes {
                        window: req.window()
                    }
                )?
                .override_redirect();

                Ok(XEvent::MapRequest(id!(req.window()), override_redirect))
            }
            Event::MapNotify(event) => Ok(XEvent::MapNotify(
                id!(event.window()),
                id!(event.event()) == self.root.id,
            )),
            Event::UnmapNotify(event) => Ok(XEvent::UnmapNotify(
                id!(event.window()),
                id!(event.event()) == self.root.id,
            )),
            Event::DestroyNotify(event) => Ok(XEvent::DestroyNotify(id!(event.window()))),
            Event::EnterNotify(event) => {
                let grab = event.mode() == x::NotifyMode::Grab;

                let id = id!(event.event());
                let abs = Point::new(event.root_x() as i32, event.root_y() as i32);
                let rel = Point::new(event.event_x() as i32, event.event_y() as i32);

                let ptrev = PointerEvent { id, abs, rel };

                Ok(XEvent::EnterNotify(ptrev, grab))
            }
            Event::LeaveNotify(event) => {
                let grab = event.mode() == x::NotifyMode::Grab;

                let id = id!(event.event());
                let abs = Point::new(event.root_x() as i32, event.root_y() as i32);
                let rel = Point::new(event.event_x() as i32, event.event_y() as i32);

                let ptrev = PointerEvent { id, abs, rel };

                Ok(XEvent::LeaveNotify(ptrev, grab))
            }
            Event::ReparentNotify(event) => Ok(XEvent::ReparentNotify(ReparentEvent {
                from_root: id!(event.event()) == self.root.id,
                parent: id!(event.parent()),
                child: id!(event.window()),
                over_red: event.override_redirect(),
            })),
            Event::PropertyNotify(event) => Ok(XEvent::PropertyNotify(PropertyEvent {
                id: id!(event.window()),
                atom: id!(event.atom()),
                time: event.time(),
                deleted: event.state() == x::Property::Delete,
            })),
            Event::KeyPress(event) => {
                let mut state = event.state();
                state.remove(x::KeyButMask::MOD2);
                Ok(XEvent::KeyPress(
                    id!(event.child()),
                    KeypressEvent {
                        mask: state.into(),
                        keycode: event.detail(),
                    },
                ))
            }
            Event::KeyRelease(_) => Ok(XEvent::KeyRelease),
            Event::ButtonPress(event) => Ok(XEvent::MouseEvent(self.do_mouse_press(event, false)?)),
            Event::ButtonRelease(event) => {
                Ok(XEvent::MouseEvent(self.do_mouse_press(event, true)?))
            }
            Event::MotionNotify(event) => Ok(XEvent::MouseEvent(self.do_mouse_motion(event)?)),
            Event::ClientMessage(event) => Ok(XEvent::ClientMessage(ClientMessageEvent {
                window: id!(event.window()),
                data: ClientMessageData::from(&event),
                type_: id!(event.r#type()),
            })),
            n => Ok(XEvent::Unknown(format!("{:?}", n))),
        }
    }

    fn get_prop_atom(&self, prop: XAtom, window: XWindowID) -> Result<Option<Property>> {
        let r = req_and_reply!(
            self.conn,
            &x::GetProperty {
                delete: false,
                window: cast!(x::Window, *window),
                property: cast!(x::Atom, *prop),
                r#type: x::ATOM_ANY,
                // start at offset 0
                long_offset: 0,
                // allow for up to 4 * MAX_LONG_LENGTH bytes of information
                long_length: MAX_LONG_LENGTH,
            }
        )?;

        if r.r#type() == x::ATOM_NONE {
            trace!("prop type is none");
            return Ok(None);
        }

        let prop_type = self.lookup_atom(id!(r.r#type()))?;
        trace!("got prop_type {}", prop_type);

        Ok(match prop_type.as_str() {
            "ATOM" => Some(Property::Atom(
                r.value()
                    .iter()
                    .map(|a| self.lookup_atom(*a).unwrap_or_else(|_| "".into()))
                    .collect::<Vec<String>>(),
            )),
            "CARDINAL" => Some(Property::Cardinal(r.value()[0])),
            "STRING" => Some(Property::String(
                String::from_utf8_lossy(r.value())
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect(),
            )),
            "UTF8_STRING" => Some(Property::UTF8String(
                String::from_utf8(r.value().to_vec())?
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect(),
            )),
            "WINDOW" => Some(Property::Window(r.value().to_vec())),
            "WM_HINTS" => Some(Property::WMHints(WmHints::try_from_bytes(r.value())?)),
            "WM_SIZE_HINTS" => Some(Property::WMSizeHints(WmSizeHints::try_from_bytes(
                r.value(),
            )?)),
            n => {
                if n == "WM_STATE" {
                    trace!("Type is WM_STATE");
                }
                match r.format() {
                    8 => Some(Property::U8List(n.into(), r.value::<u8>().into())),
                    16 => Some(Property::U16List(n.into(), r.value::<u16>().into())),
                    32 => Some(Property::U32List(n.into(), r.value::<u32>().into())),
                    n => {
                        return Err(XError::InvalidPropertyData(format!(
                            "received format {}",
                            n
                        )))
                    }
                }
            }
        })
    }
}

impl<S: ConnStatus> fmt::Debug for XCBConn<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XCBConn")
            .field("root", &self.root)
            .field("idx", &self.idx)
            .field("randr", &self.randr_base)
            //.field("atoms", &self.atoms)
            .field("cursor", &self.cursor)
            .field("mousemode", &self.mousemode)
            .finish()
    }
}

impl From<xcb::ConnError> for XError {
    fn from(e: xcb::ConnError) -> XError {
        XError::Connection(e.to_string())
    }
}

impl From<xcb::ProtocolError> for XError {
    fn from(e: xcb::ProtocolError) -> XError {
        XError::Protocol(e.to_string())
    }
}

impl From<xcb::Error> for XError {
    fn from(e: xcb::Error) -> XError {
        match e {
            xcb::Error::Connection(e) => e.into(),
            xcb::Error::Protocol(e) => e.into(),
        }
    }
}

impl From<&x::ClientMessageEvent> for ClientMessageData {
    fn from(event: &x::ClientMessageEvent) -> Self {
        let data = event.data();
        match data {
            x::ClientMessageData::Data8(dat) => Self::U8(dat),
            x::ClientMessageData::Data16(dat) => Self::U16(dat),
            x::ClientMessageData::Data32(dat) => Self::U32(dat),
        }
    }
}
