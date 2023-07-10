use std::cell::{Cell, RefCell};
use std::fmt;

use x11rb::rust_connection::RustConnection;
use x11rb::connection::Connection;
use x11rb::protocol::{
    Event,
    xproto::{self,
        Screen, Depth,
        ConnectionExt as XConnectionExt
    },
    randr::ConnectionExt as RConnectionExt,
    randr,
};

use tracing::instrument;
use tracing::{trace, debug,};

use strum::*;

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
        ClientMessageEvent,
        ClientMessageData,
    },
    property::{
        Property,
        WmHints,
        WmSizeHints,
    },
    input::KeyButMask,
    cursor,
};
use crate::types::{
    Point, Geometry,
};
use crate::keybinds::ButtonIndex;

mod xconn;
mod convert;

#[cfg(test)]
mod tests;

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
/// ```rust
/// use toaruwm::x::x11rb::X11RBConn;
/// 
/// let mut conn = X11RBConn::connect().expect("Connection error");
/// 
/// conn.init().expect("Could not initialize");
/// 
/// /* or: */
/// let mut conn = X11RBConn::new().expect("Connection error");
/// ```
/// 
/// [1]: crate::x::core::XConn
/// [2]: crate::manager::WindowManager
pub struct X11RBConn {
    conn: RustConnection,
    root: XWindow,
    idx: usize,
    randr_base: u8,
    atoms: RefCell<Atoms>, // wrap in RefCell for interior mutability
    cursor: u32,
    mousemode: Cell<Option<ButtonIndex>>, // ditto
}

impl X11RBConn {
    /// Connects and initializes a new Connection.
    pub fn new() -> Result<Self> {
        let mut conn = Self::connect()?;
        conn.init()?;

        Ok(conn)
    }

    /// Connect to the X server and allocate a new Connection.
    pub fn connect() -> Result<Self> {

        // initialize xcb connection
        let (conn, idx) = x11rb::connect(None)?;
        trace!("Connected to x server, got preferred screen {}", idx);
        // wrap it in an ewmh connection just for fun

        // initialize our atom handler
        let atoms = RefCell::new(Atoms::new());
        let cursor = conn.generate_id()?;

        Ok(Self {
            conn,
            root: XWindow::zeroed(),
            idx,
            randr_base: 0,
            atoms,
            cursor,
            mousemode: Cell::new(None),
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
    /// [1]: crate::x::Atom;
    pub fn init(&mut self) -> Result<()> {

        // validate randr version
        let res = self.conn.randr_query_version(
            RANDR_MAJ, RANDR_MIN
        )?.reply()?;

        let (maj, min) = (res.major_version, res.minor_version);

        trace!("Got randr version {}.{}", maj, min);

        if maj != RANDR_MAJ || min < RANDR_MIN {
            return Err(XError::RandrError(
                format!(
                    "Received randr version {}.{}, requires v{}.{} or higher",
                    maj, min, RANDR_MAJ, RANDR_MIN
                )
            ))
        }

        // get root window id
        self.root = match self.conn.setup().roots
            .iter()
            .nth(self.idx as usize)
        {
            Some(screen) => {
                let id = screen.root;
                let geom = self.get_geometry(id)?;
                XWindow::with_data(id, geom)
            },
            None => return Err(XError::NoScreens),
        };
        trace!("Got root: {:?}", self.root);

        // initialize randr and get its event mask
        self.randr_base = self.conn.query_extension(
            randr::X11_EXTENSION_NAME.as_bytes()
        )?
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
                self.conn.intern_atom(
                    false, atom.as_ref().as_bytes()
                )?
            ));
        }

        let atoms = self.atoms.get_mut();

        // then get replies
        for (name, cookie) in atomvec {
            atoms.insert(
                &name,
                cookie.reply()?.atom
            );
        }

        // initialize cursor and set it for the root screen
        self.create_cursor(cursor::LEFT_PTR)?;
        self.set_cursor(self.root.id)?;

        Ok(())
    }

    /// Adds an atom to internal atom storage.
    pub fn add_atom<S: AsRef<str>>(&mut self, name: S, atom: XAtom) {
        self.atoms.get_mut().insert(name.as_ref(), atom);
    }

    /// Returns a reference to its internal atom storage.
    pub fn atoms(&self) -> &Atoms {
        // SAFETY: returns an immutable reference
        unsafe {&*self.atoms.as_ptr()}
    }

    /// Exposes `X11RBConn`'s internal connection.
    pub fn conn(&self) -> &RustConnection {
        &self.conn
    }

    pub fn create_cursor(&mut self, glyph: u16) -> Result<()> {
        trace!("creating cursor");

        let fid = self.conn.generate_id()?;
        self.conn.open_font(
            fid, "cursor".as_bytes()
        )?.check()?;

        let cid = self.conn.generate_id()?;
        self.conn.create_glyph_cursor(
            cid,
            fid, fid,
            glyph, glyph + 1,
            0, 0, 0,
            0xffff, 0xffff, 0xffff,
        )?.check()?;

        self.cursor = cid;
        Ok(())
    }

    pub fn set_cursor(&self, window: XWindowID) -> Result<()> {
        use x11rb::protocol::xproto::ChangeWindowAttributesAux;

        trace!("setting cursor for {}", window);

        self.conn.change_window_attributes(
            window,
            &ChangeWindowAttributesAux::new().cursor(self.cursor)
        )?.check()?;

        Ok(())
    }

    pub(crate) fn check_win(&self) -> Result<XWindowID> {
        self.create_window(
            WindowClass::CheckWin, 
            Geometry::new(0, 0, 1, 1,),
            false,
        )
    }

    pub(crate) fn screen(&self, idx: usize) -> Result<&Screen>  {
        let roots = &self.conn.setup().roots;

        if idx >= roots.len() {
            Err(XError::InvalidScreen)
        } else {
            Ok(&roots[idx])
        }
    }

    pub(crate) fn depth<'a>(&self, screen: &'a Screen) -> Result<&'a Depth> {
        Ok(screen.allowed_depths.iter()
            .max_by(|x, y| x.depth.cmp(&y.depth))
            .ok_or(XError::RequestError("No allowed depths for screen"))?)
    }

    pub(crate) fn visual_type(&self, depth: &Depth) -> Result<xproto::Visualtype> {
        Ok(*(depth.visuals.iter()
            .find(|v| v.class == xproto::VisualClass::TRUE_COLOR)
            .ok_or(XError::RequestError("Could not get true color visualtype")))?)
    }
    
    #[instrument(target="xcbconn", level="trace", skip(self))]
    fn process_raw_event(&self, event: Event) -> Result<XEvent> {
        match event {
            //* RandR events
            // todo: use the data provided if needed
            Event::RandrNotify(_) => Ok(XEvent::RandrNotify),
            Event::RandrScreenChangeNotify(_) => Ok(XEvent::ScreenChange),

            //* Core X protocol events
            Event::ConfigureNotify(event) => {
                if event.event == self.root.id {
                    trace!("Top level window configuration");
                }
                Ok(XEvent::ConfigureNotify(ConfigureEvent {
                    id: event.window,
                    geom: Geometry {
                        x: event.x as i32,
                        y: event.y as i32,
                        height: event.height as i32,
                        width: event.width as i32
                    },
                    is_root: event.window == self.root.id,
                }))
            }
            Event::ConfigureRequest(req) => {
                use StackMode::*;
                use xproto::{
                    StackMode as XStackMode,
                    ConfigWindow as CWMask,
                };

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
                } else {None};
                let y = if vmask.contains(CWMask::Y) {
                    Some(req.y as i32)
                } else {None};
                let height = if vmask.contains(CWMask::HEIGHT) {
                    Some(req.height as u32)
                } else {None};
                let width = if vmask.contains(CWMask::WIDTH) {
                    Some(req.width as u32)
                } else {None};
                let stack_mode = if vmask.contains(CWMask::STACK_MODE) {
                    match req.stack_mode {
                        XStackMode::ABOVE => Some(Above),
                        XStackMode::BELOW => Some(Below),
                        XStackMode::TOP_IF => Some(TopIf),
                        XStackMode::BOTTOM_IF => Some(BottomIf),
                        XStackMode::OPPOSITE => Some(Opposite),
                        _ => return Err(XError::ConversionError)
                    }
                } else {None};
                let sibling = if vmask.contains(CWMask::SIBLING) {
                    Some(req.sibling)
                } else {None};

                Ok(XEvent::ConfigureRequest(ConfigureRequestData {
                    id,
                    parent,
                    sibling,
                    x, y, height, width,
                    stack_mode,
                    is_root,
                }))
            }
            Event::MapRequest(req) => {
                let override_redirect = self.conn.get_window_attributes(req.window)?
                    .reply()?.override_redirect;

                Ok(XEvent::MapRequest(req.window, override_redirect))
            }
            Event::MapNotify(event) => {
                Ok(XEvent::MapNotify(event.window))
            }
            Event::UnmapNotify(event) => {
                Ok(XEvent::UnmapNotify(event.window))
            }
            Event::DestroyNotify(event) => {
                Ok(XEvent::DestroyNotify(event.window))
            }
            Event::EnterNotify(event) => {
                let grab = event.mode == xproto::NotifyMode::GRAB;

                let id = event.event;
                let abs = Point::new(event.root_x as i32, event.root_y as i32);
                let rel = Point::new(event.event_x as i32, event.event_y as i32);

                let ptrev = PointerEvent {id, abs, rel};

                Ok(XEvent::EnterNotify(ptrev, grab))
            }
            Event::LeaveNotify(event) => {
                let grab = event.mode == xproto::NotifyMode::GRAB;

                let id = event.event;
                let abs = Point::new(event.root_x as i32, event.root_y as i32);
                let rel = Point::new(event.event_x as i32, event.event_y as i32);

                let ptrev = PointerEvent {id, abs, rel};

                Ok(XEvent::LeaveNotify(ptrev, grab))
            }
            Event::ReparentNotify(event) => {
                Ok(XEvent::ReparentNotify(ReparentEvent {
                    event: event.event,
                    parent: event.parent,
                    child: event.window,
                    over_red: event.override_redirect,
                }))
            }
            Event::PropertyNotify(event) => {
                Ok(XEvent::PropertyNotify(PropertyEvent {
                    id: event.window,
                    atom: event.atom,
                    time: event.time,
                    deleted: event.state == xproto::Property::DELETE,
                }))
            }
            Event::KeyPress(event) => {
                let mut mask = KeyButMask::from(event.state);
                // filter out mod2
                mask.remove(KeyButMask::from(xproto::KeyButMask::MOD2));
                Ok(XEvent::KeyPress(event.child, KeypressEvent {
                    mask: mask.modmask(),
                    keycode: event.detail,
                }))
            }
            Event::KeyRelease(_) => {
                Ok(XEvent::KeyRelease)
            }
            Event::ButtonPress(event) => {
                Ok(XEvent::MouseEvent(self.do_mouse_press(event, false)?))
            }
            Event::ButtonRelease(event) => {
                Ok(XEvent::MouseEvent(self.do_mouse_press(event, true)?))
            }
            Event::MotionNotify(event) => {
                Ok(XEvent::MouseEvent(self.do_mouse_motion(event)?))
            }
            Event::ClientMessage(event) => {
                Ok(XEvent::ClientMessage(ClientMessageEvent{
                    window: event.window,
                    data: ClientMessageData::from(&event),
                    type_: event.type_,
                }))
            }
            unk => {
                debug!("got unknown event {:?}", unk);
                Ok(XEvent::Unknown(format!("{:?}", unk)))
            }
        }
    }

    fn get_prop_atom(&self, prop: XAtom, window: XWindowID) -> Result<Option<Property>> {
        let r = self.conn.get_property(
            false,
            window,
            prop,
            xproto::AtomEnum::ANY,
            // start at offset 0
            0, 
            // allow for up to 4 * MAX_LONG_LENGTH bytes of information
            MAX_LONG_LENGTH,
        )?.reply()?;

        if r.type_ == x11rb::NONE {
            trace!("prop type is none");
            return Ok(None)
        }

        let prop_type = self.lookup_atom(r.type_)?;
        trace!("got prop_type {}", prop_type);

        Ok(match prop_type.as_str() {
            "ATOM" => Some(Property::Atom({
                r.value32().unwrap()
                    .map(|a| self.lookup_atom(a).unwrap_or_else(|_| "".into()))
                    .collect()
            })),
            "CARDINAL" => Some(Property::Cardinal(
                r.value32().unwrap().nth(0).unwrap()
            )),
            "STRING" => Some(Property::String(
                String::from_utf8_lossy(&r.value)
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect()
            )),
            "UTF8_STRING" => Some(Property::UTF8String(
                String::from_utf8(r.value)?
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect()
            )),
            "WINDOW" => Some(Property::Window(r.value32().unwrap().collect())),
            "WM_HINTS" => Some(Property::WMHints(
                WmHints::try_from_bytes(
                    &r.value32().ok_or(
                        XError::ConversionError
                    )?.collect::<Vec<u32>>()
                )?
            )),
            "WM_SIZE_HINTS" => Some(Property::WMSizeHints(
                WmSizeHints::try_from_bytes(
                    &r.value32().ok_or(
                        XError::ConversionError
                    )?.collect::<Vec<u32>>())?
            )),
            n => {
                if n == "WM_STATE" {
                    trace!("Type is WM_STATE");
                }
                match r.format {
                    8 => Some(Property::U8List(
                        n.into(),
                        r.value8().unwrap().collect()
                    )),
                    16 => Some(Property::U16List(
                        n.into(),
                        r.value16().unwrap().collect()
                    )),
                    32 => Some(Property::U32List(
                        n.into(),
                        r.value32().unwrap().collect()
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

impl fmt::Debug for X11RBConn {
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
            IoError(e) => {
                XError::Connection(e.to_string())
            }
            other => {
                XError::Protocol(other.to_string())
            }
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
            8  => Self::U8(event.data.as_data8()),
            16 => Self::U16(event.data.as_data16()),
            32 => Self::U32(event.data.as_data32()),
            _ => unreachable!()
        }
    }
}