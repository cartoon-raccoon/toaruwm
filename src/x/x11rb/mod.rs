use core::marker::PhantomData;

use std::cell::{Cell, RefCell};
use std::fmt;

use x11rb::connection::Connection;
use x11rb::protocol::{
    randr,
    randr::ConnectionExt as RConnectionExt,
    xproto::{self, ConnectionExt as XConnectionExt, Depth, Screen},
    Event,
};
use x11rb::rust_connection::RustConnection;

use tracing::trace;

use strum::*;

use super::{
    atom::Atom,
    core::{Result, StackMode, WindowClass, XAtom, XConn, XError, XWindow, XWindowID},
    cursor,
    event::{
        ClientMessageData, ClientMessageEvent, ConfigureEvent, ConfigureRequestData, KeypressEvent,
        PointerEvent, PropertyEvent, ReparentEvent, XEvent,
    },
    input::KeyButMask,
    property::{Property, WmHints, WmSizeHints},
    Atoms, ConnStatus, Initialized, Uninitialized,
};
use crate::bindings::ButtonIndex;
use crate::types::{Geometry, Point};

mod convert;
mod xconn;

const MAX_LONG_LENGTH: u32 = 1024;

const RANDR_MAJ: u32 = 1;
const RANDR_MIN: u32 = 4;

/// A connection to an X server, backed by the x11rb library.
///
/// This is a very simple connection to the X server
/// and is completely synchronous, despite the async capabilities
/// of the underlying library.
///
/// It implements [XConn][1] and thus can be used with a
/// [WindowManager][2].
///
/// # Usage
///
/// ```no_run
/// use toaruwm::x::x11rb::X11RBConn;
///
/// let conn = X11RBConn::connect().expect("Connection error");
/// let mut conn = conn.init().expect("Could not initialize");
///
/// /* or: */
/// let mut conn = X11RBConn::new().expect("Connection error");
/// ```
///
/// [1]: crate::x::core::XConn
/// [2]: crate::manager::WindowManager
pub struct X11RBConn<S: ConnStatus> {
    conn: RustConnection,
    root: XWindow,
    idx: usize,
    randr_base: u8,
    atoms: RefCell<Atoms>, // wrap in RefCell for interior mutability
    cursor: u32,
    mousemode: Cell<Option<ButtonIndex>>, // ditto
    _marker: PhantomData<S>,
}

impl X11RBConn<Uninitialized> {
    /// Connect to the X server and allocate a new Connection.
    pub fn connect() -> Result<Self> {
        // initialize xcb connection
        let (conn, idx) = x11rb::connect(None)?;
        trace!("Connected to x server, got preferred screen {}", idx);
        // wrap it in an ewmh connection just for fun

        // initialize our atom handler
        let atoms = RefCell::new(Atoms::new());

        Ok(Self {
            conn,
            root: XWindow::zeroed(),
            idx,
            randr_base: 0,
            atoms,
            cursor: 0,
            mousemode: Cell::new(None),
            _marker: PhantomData,
        })
    }

    /// Initializes the connection.
    ///
    /// It does the following:
    ///
    /// - Verifies the randr version is compatible.
    /// - Initializes the randr extension.
    /// - Initializes the root window and its dimensions.
    /// - Interns all known [atoms][1].
    /// - Creates and sets the cursor.
    ///
    /// [1]: crate::x::atom::Atom
    pub fn init(mut self) -> Result<X11RBConn<Initialized>> {
        // validate randr version
        let res = self
            .conn
            .randr_query_version(RANDR_MAJ, RANDR_MIN)?
            .reply()?;

        let (maj, min) = (res.major_version, res.minor_version);

        trace!("Got randr version {}.{}", maj, min);

        if maj != RANDR_MAJ || min < RANDR_MIN {
            return Err(XError::RandrError(format!(
                "Received randr version {}.{}, requires v{}.{} or higher",
                maj, min, RANDR_MAJ, RANDR_MIN
            )));
        }

        // get root window id
        let root = match self.conn.setup().roots.get(self.idx) {
            Some(screen) => {
                let id = screen.root;
                let geom = self.get_geometry_inner(id)?;
                XWindow::with_data(id, geom)
            }
            None => return Err(XError::NoScreens),
        };
        trace!("Got root: {:?}", self.root);

        // initialize randr and get its event mask
        let randr_base = self
            .conn
            .query_extension(randr::X11_EXTENSION_NAME.as_bytes())?
            //.ok_or_else(|| XError::RandrError("could not load randr".into()))?
            .reply()?
            .first_event;

        trace!("Got randr_base {}", self.randr_base);

        let atomcount = Atom::iter().count();
        let mut atomvec = Vec::with_capacity(atomcount);

        // intern all known atoms

        // get cookies for all first
        for atom in Atom::iter() {
            atomvec.push((
                atom.to_string(),
                self.conn.intern_atom(false, atom.as_ref().as_bytes())?,
            ));
        }

        let atoms = self.atoms.get_mut();

        // then get replies
        for (name, cookie) in atomvec {
            atoms.insert(&name, cookie.reply()?.atom);
        }

        // initialize cursor and set it for the root screen
        let cursor = self.create_cursor_inner(cursor::LEFT_PTR)?;
        self.set_cursor_inner(root.id, cursor)?;

        Ok(X11RBConn {
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

impl<S: ConnStatus> X11RBConn<S> {
    #[inline]
    pub(crate) fn get_geometry_inner(&self, window: XWindowID) -> Result<Geometry> {
        trace!("Getting geometry for window {}", window);

        // send the request and grab its reply
        Ok(self.conn.get_geometry(window)?.reply().map(|ok| Geometry {
            // map the ok result into a Geometry
            x: ok.x as i32,
            y: ok.y as i32,
            height: ok.height as i32,
            width: ok.width as i32,
        })?)
    }
    #[inline]
    pub(crate) fn create_cursor_inner(&mut self, glyph: u16) -> Result<u32> {
        trace!("creating cursor");

        let fid = self.conn.generate_id()?;
        self.conn.open_font(fid, "cursor".as_bytes())?.check()?;

        let cid = self.conn.generate_id()?;
        self.conn
            .create_glyph_cursor(
                cid,
                fid,
                fid,
                glyph,
                glyph + 1,
                0,
                0,
                0,
                0xffff,
                0xffff,
                0xffff,
            )?
            .check()?;

        Ok(cid)
    }
    #[inline]
    pub(crate) fn set_cursor_inner(&self, window: XWindowID, cursor: u32) -> Result<()> {
        use x11rb::protocol::xproto::ChangeWindowAttributesAux;

        trace!("setting cursor for {}", window);

        self.conn
            .change_window_attributes(window, &ChangeWindowAttributesAux::new().cursor(cursor))?
            .check()?;

        Ok(())
    }
}

impl X11RBConn<Initialized> {
    /// Shortcut static method for directly creating
    /// an initialized connection.
    pub fn new() -> Result<Self> {
        X11RBConn::connect()?.init()
    }

    /// Adds an atom to internal atom storage.
    pub fn add_atom<S: AsRef<str>>(&mut self, name: S, atom: XAtom) {
        self.atoms.get_mut().insert(name.as_ref(), atom);
    }

    /// Returns a reference to its internal atom storage.
    pub fn atoms(&self) -> &Atoms {
        // SAFETY: returns an immutable reference
        unsafe { &*self.atoms.as_ptr() }
    }

    /// Exposes `X11RBConn`'s internal connection.
    pub fn conn(&self) -> &RustConnection {
        &self.conn
    }

    /// Allocates a new cursor in the X server.
    pub fn create_cursor(&mut self, glyph: u16) -> Result<()> {
        self.cursor = self.create_cursor_inner(glyph)?;
        Ok(())
    }

    /// Sets a cursor for the given window.
    pub fn set_cursor(&self, window: XWindowID) -> Result<()> {
        self.set_cursor_inner(window, self.cursor)
    }

    pub(crate) fn check_win(&self) -> Result<XWindowID> {
        self.create_window(WindowClass::CheckWin, Geometry::new(0, 0, 1, 1), false)
    }

    pub(crate) fn screen(&self, idx: usize) -> Result<&Screen> {
        let roots = &self.conn.setup().roots;

        if idx >= roots.len() {
            Err(XError::InvalidScreen)
        } else {
            Ok(&roots[idx])
        }
    }

    pub(crate) fn depth<'a>(&self, screen: &'a Screen) -> Result<&'a Depth> {
        screen
            .allowed_depths
            .iter()
            .max_by(|x, y| x.depth.cmp(&y.depth))
            .ok_or(XError::RequestError("No allowed depths for screen"))
    }

    pub(crate) fn visual_type(&self, depth: &Depth) -> Result<xproto::Visualtype> {
        Ok(*(depth
            .visuals
            .iter()
            .find(|v| v.class == xproto::VisualClass::TRUE_COLOR)
            .ok_or(XError::RequestError("Could not get true color visualtype")))?)
    }

    //#[instrument(target = "x11rbconn", level = "trace", skip(self))]
    fn process_raw_event(&self, event: Event) -> Result<XEvent> {
        match event {
            //* RandR events
            // todo: use the data provided if needed
            Event::RandrNotify(_) => Ok(XEvent::RandrNotify),
            Event::RandrScreenChangeNotify(_) => Ok(XEvent::ScreenChange),

            //* Core X protocol events
            Event::ConfigureNotify(event) => Ok(XEvent::ConfigureNotify(ConfigureEvent {
                from_root: event.event == self.root.id,
                id: event.window,
                geom: Geometry {
                    x: event.x as i32,
                    y: event.y as i32,
                    height: event.height as i32,
                    width: event.width as i32,
                },
                is_root: event.window == self.root.id,
            })),
            Event::ConfigureRequest(req) => {
                use xproto::{ConfigWindow as CWMask, StackMode as XStackMode};
                use StackMode::*;

                // extract window ids
                let id = req.window;
                let parent = req.parent;
                let is_root = id == self.root.id;
                if parent == self.root.id {
                    trace!("Top level window configuration request");
                }

                // extract relevant values using the value mask
                let vmask = req.value_mask;
                let x = if vmask.contains(CWMask::X) {
                    Some(req.x as i32)
                } else {
                    None
                };
                let y = if vmask.contains(CWMask::Y) {
                    Some(req.y as i32)
                } else {
                    None
                };
                let height = if vmask.contains(CWMask::HEIGHT) {
                    Some(req.height as u32)
                } else {
                    None
                };
                let width = if vmask.contains(CWMask::WIDTH) {
                    Some(req.width as u32)
                } else {
                    None
                };
                let stack_mode = if vmask.contains(CWMask::STACK_MODE) {
                    match req.stack_mode {
                        XStackMode::ABOVE => Some(Above),
                        XStackMode::BELOW => Some(Below),
                        XStackMode::TOP_IF => Some(TopIf),
                        XStackMode::BOTTOM_IF => Some(BottomIf),
                        XStackMode::OPPOSITE => Some(Opposite),
                        _ => return Err(XError::ConversionError),
                    }
                } else {
                    None
                };
                let sibling = if vmask.contains(CWMask::SIBLING) {
                    Some(req.sibling)
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
                let override_redirect = self
                    .conn
                    .get_window_attributes(req.window)?
                    .reply()?
                    .override_redirect;

                Ok(XEvent::MapRequest(req.window, override_redirect))
            }
            Event::MapNotify(event) => Ok(XEvent::MapNotify(
                event.window, event.event == self.root.id
            )),
            Event::UnmapNotify(event) => Ok(XEvent::UnmapNotify(
                event.window, event.event == self.root.id
            )),
            Event::DestroyNotify(event) => Ok(XEvent::DestroyNotify(event.window)),
            Event::EnterNotify(event) => {
                let grab = event.mode == xproto::NotifyMode::GRAB;

                let id = event.event;
                let abs = Point::new(event.root_x as i32, event.root_y as i32);
                let rel = Point::new(event.event_x as i32, event.event_y as i32);

                let ptrev = PointerEvent { id, abs, rel };

                Ok(XEvent::EnterNotify(ptrev, grab))
            }
            Event::LeaveNotify(event) => {
                let grab = event.mode == xproto::NotifyMode::GRAB;

                let id = event.event;
                let abs = Point::new(event.root_x as i32, event.root_y as i32);
                let rel = Point::new(event.event_x as i32, event.event_y as i32);

                let ptrev = PointerEvent { id, abs, rel };

                Ok(XEvent::LeaveNotify(ptrev, grab))
            }
            Event::ReparentNotify(event) => Ok(XEvent::ReparentNotify(ReparentEvent {
                from_root: event.event == self.root.id,
                parent: event.parent,
                child: event.window,
                over_red: event.override_redirect,
            })),
            Event::PropertyNotify(event) => Ok(XEvent::PropertyNotify(PropertyEvent {
                id: event.window,
                atom: event.atom,
                time: event.time,
                deleted: event.state == xproto::Property::DELETE,
            })),
            Event::KeyPress(event) => {
                let mut mask = KeyButMask::from(event.state);
                // filter out mod2
                mask.remove(KeyButMask::from(xproto::KeyButMask::MOD2));
                Ok(XEvent::KeyPress(
                    event.child,
                    KeypressEvent {
                        mask: mask.modmask(),
                        keycode: event.detail,
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
                window: event.window,
                data: ClientMessageData::from(&event),
                type_: event.type_,
            })),
            unk => Ok(XEvent::Unknown(format!("{:?}", unk))),
        }
    }

    fn get_prop_atom(&self, prop: XAtom, window: XWindowID) -> Result<Option<Property>> {
        let r = self
            .conn
            .get_property(
                false,
                window,
                prop,
                xproto::AtomEnum::ANY,
                // start at offset 0
                0,
                // allow for up to 4 * MAX_LONG_LENGTH bytes of information
                MAX_LONG_LENGTH,
            )?
            .reply()?;

        if r.type_ == x11rb::NONE {
            trace!("prop type is none");
            return Ok(None);
        }

        let prop_type = self.lookup_atom(r.type_)?;
        trace!("got prop_type {}", prop_type);

        Ok(match prop_type.as_str() {
            "ATOM" => Some(Property::Atom({
                r.value32()
                    .unwrap()
                    .map(|a| self.lookup_atom(a).unwrap_or_else(|_| "".into()))
                    .collect()
            })),
            "CARDINAL" => Some(Property::Cardinal(r.value32().unwrap().next().unwrap())),
            "STRING" => Some(Property::String(
                String::from_utf8_lossy(&r.value)
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect(),
            )),
            "UTF8_STRING" => Some(Property::UTF8String(
                String::from_utf8(r.value)?
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect(),
            )),
            "WINDOW" => Some(Property::Window(r.value32().unwrap().collect())),
            "WM_HINTS" => Some(Property::WMHints(WmHints::try_from_bytes(
                &r.value32()
                    .ok_or(XError::ConversionError)?
                    .collect::<Vec<u32>>(),
            )?)),
            "WM_SIZE_HINTS" => Some(Property::WMSizeHints(WmSizeHints::try_from_bytes(
                &r.value32()
                    .ok_or(XError::ConversionError)?
                    .collect::<Vec<u32>>(),
            )?)),
            n => {
                if n == "WM_STATE" {
                    trace!("Type is WM_STATE");
                }
                match r.format {
                    8 => Some(Property::U8List(n.into(), r.value8().unwrap().collect())),
                    16 => Some(Property::U16List(n.into(), r.value16().unwrap().collect())),
                    32 => Some(Property::U32List(n.into(), r.value32().unwrap().collect())),
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

impl<S: ConnStatus + fmt::Debug> fmt::Debug for X11RBConn<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("X11RBConn")
            .field("root", &self.root)
            .field("idx", &self.idx)
            .field("randr", &self.randr_base)
            //.field("atoms", &self.atoms)
            .field("cursor", &self.cursor)
            .field("mousemode", &self.mousemode)
            .finish()
    }
}

use std::io::Error;

impl From<Error> for XError {
    fn from(_: Error) -> XError {
        XError::ConversionError
    }
}

use x11rb::errors;

impl From<errors::ConnectionError> for XError {
    fn from(e: errors::ConnectionError) -> XError {
        use errors::ConnectionError::*;
        match e {
            UnknownError | InsufficientMemory | FdPassingFailed => {
                XError::Connection(e.to_string())
            }
            IoError(e) => XError::Connection(e.to_string()),
            other => XError::Protocol(other.to_string()),
        }
    }
}

impl From<errors::ConnectError> for XError {
    fn from(e: errors::ConnectError) -> XError {
        XError::Connection(e.to_string())
    }
}

impl From<errors::ReplyError> for XError {
    fn from(e: errors::ReplyError) -> XError {
        XError::Protocol(e.to_string())
    }
}

impl From<errors::ReplyOrIdError> for XError {
    fn from(e: errors::ReplyOrIdError) -> XError {
        if let errors::ReplyOrIdError::ConnectionError(e) = e {
            e.into()
        } else {
            XError::ServerError(e.to_string())
        }
    }
}

impl From<&xproto::ClientMessageEvent> for ClientMessageData {
    fn from(event: &xproto::ClientMessageEvent) -> Self {
        let format = event.format;
        match format {
            8 => Self::U8(event.data.as_data8()),
            16 => Self::U16(event.data.as_data16()),
            32 => Self::U32(event.data.as_data32()),
            _ => unreachable!(),
        }
    }
}
