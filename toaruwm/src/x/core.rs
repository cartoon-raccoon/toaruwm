//! Core types and traits for interfacing with the X server.
//!
//! Core functionality of ToaruWM's interface with the X server.
//!
//! This module defines core types and traits used throughout this
//! crate for directly interacting with the X server.

use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, Not,
    Deref, DerefMut,
};
use std::str::FromStr;
use std::fmt::{self, Display};

use thiserror::Error;
use tracing::{debug, error, warn};

use crate::bindings::{Keybind, Mousebind};

use super::{
    atom::{Atom, AUTO_FLOAT_WINDOW_TYPES, UNMANAGED_WINDOW_TYPES},
    event::{ClientMessageEvent, XEvent},
    input::KeyButMask,
    property::*,
};
use crate::core::Screen;
use crate::types::{ClientAttrs, ClientConfig, Geometry, XWinProperties};

//* ========== X WINDOW THINGS ========== *//

/// General constant for expressing None when passing X IDs.
pub const XID_NONE: Xid = Xid::zero();

/// Wrapper type to represent IDs used by the X server.
/// 
/// This is used by the server to identify all sorts
/// of X window resources, including windows and atoms.
/// 
/// You can create an Xid from a `u32`:
/// 
/// ```rust
/// use toaruwm::x::Xid;
/// 
/// let id = Xid::from(69);
/// let val = id.val();
/// 
/// assert_eq!(val, 69);
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Xid(pub(crate) u32);

impl Xid {
    /// Return an Xid set to 0.
    pub const fn zero() -> Self {
        Xid(0)
    }

    /// Returns the internal value of the Xid.
    pub const fn val(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Xid {
    fn from(f: u32) -> Xid {
        Xid(f)
    }
}

impl Display for Xid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Xid({})", self.0)
    }
}

impl Deref for Xid {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.0
    }
}

impl DerefMut for Xid {
    fn deref_mut(&mut self) -> &mut u32 {
        &mut self.0
    }
}

/// An X server ID for a given window.
pub type XWindowID = Xid;

/// An X Atom.
pub type XAtom = Xid;

/// A marker trait to signal that a type can be treated as a bitmask.
///
/// This means that the type supports bitmask operations such as
/// bitwise AND, bitwise OR, bitwise NOT, etc.
pub trait BitMask
where
    Self: BitAnd + BitOr + Not + BitAndAssign + BitOrAssign + Sized,
{
}

// Blanket implementation for Bitmask
impl<T> BitMask for T where T: BitAnd + BitOr + Not + BitAndAssign + BitOrAssign + Sized {}

/// Window stacking modes defined by the X Protocol.
/// 
/// Each variant may carry a `sibling` window ID, that
/// changes the semantics of the `StackMode`.
/// 
/// The exact semantics of this difference are explained
/// [here](https://tronche.com/gui/x/xlib/window/configure.html).
#[derive(Clone, Copy, Debug)]
pub enum StackMode {
    /// Stack the window at the top of the stack.
    /// 
    /// If a sibling is specified, the window is instead stacked
    /// just above the specified sibling.
    Above(Option<XWindowID>),
    /// Stack the window at the bottom of the stack.
    /// 
    /// If a sibling is specified, the window is instead stacked
    /// just below the specified sibling.
    Below(Option<XWindowID>),
    /// If any sibling occludes the window, the window
    /// is stacked at the top of the stack.
    /// 
    /// If a sibling is specified, then the window is
    /// stacked only if the sibling occludes it.
    TopIf(Option<XWindowID>),
    /// If the window occludes any sibling, the window
    /// is stacked at the bottom of the stack.
    /// 
    /// If a sibling is specified, then the window is
    /// stacked only if it occludes any sibling.
    BottomIf(Option<XWindowID>),
    /// TODO
    Opposite(Option<XWindowID>),
}

/// Reply to a pointer query.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PointerQueryReply {
    /// If the pointer is on the current screen.
    pub same_screen: bool,
    /// The root window.
    pub root: XWindowID,
    /// The child containing the pointer.
    pub child: XWindowID,
    /// The x-coordinate of the pointer relative to the root window.
    pub root_x: i32,
    /// The y-coordinate of the pointer relative to the root window.
    pub root_y: i32,
    /// The x-coordinate of the pointer relative to the child window.
    pub win_x: i32,
    /// The y-coordinate of the pointer relative to the child window.
    pub win_y: i32,
    /// The logical state of the buttons and modkeys.
    pub mask: KeyButMask,
}

/// Representation of an X window with additional data (geometry).
#[derive(Debug, Clone, Copy)]
pub struct XWindow {
    /// The X ID assigned to the window.
    pub id: XWindowID,
    /// The geometry of the window as stored on the X server.
    pub geom: Geometry,
}

impl XWindow {
    /// Creates an `XWindow` with all fields zeroed.
    pub fn zeroed() -> Self {
        XWindow {
            id: Xid(0),
            geom: Geometry::zeroed(),
        }
    }

    /// Creates an `XWindow` with the given data.
    pub fn with_data(id: XWindowID, geom: Geometry) -> Self {
        XWindow { id, geom }
    }
}

impl PartialEq for XWindow {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<XWindowID> for XWindow {
    fn from(from: XWindowID) -> Self {
        Self {
            id: from,
            geom: Geometry {
                x: 0,
                y: 0,
                height: 0,
                width: 0,
            },
        }
    }
}

/// The type of window that you want to create.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowClass {
    /// An invisible window to make API calls
    CheckWin,
    /// A window that only accepts input
    InputOnly,
    /// A regular window. The Atom provided should be a valid
    /// _NET_WM_WINDOW_TYPE, and the u32 is the border width
    /// in pixels.
    ///
    /// This should be the value returned by a
    /// [`RuntimeConfig`](crate::manager::RuntimeConfig)'s
    /// `border_px` method.
    InputOutput(Atom, u32),
    //todo: add additional variants for new window types?
}

impl XWindow {
    /// Sets the geometry using an XConn object.
    ///
    /// This method requests the X server directly for the
    /// geometry of the window and updates its internal fields
    /// accordingly.
    pub fn set_geometry_conn<X: XConn>(&mut self, conn: &X) {
        match conn.get_geometry(self.id) {
            Ok(geom) => {
                debug!(
                    "Updating geometry (conn): x: {}, y: {}, h: {}, w: {}",
                    geom.x, geom.y, geom.height, geom.width
                );
                self.geom = geom;
            }

            Err(e) => {
                error!("{}", e);
            }
        }
    }

    /// Sets the geometry using a provided Geometry.
    ///
    /// Note that this does not update the geometry as tracked by
    /// the X server, and so a request should be made to the server
    /// to update the geometry there as well.
    pub fn set_geometry(&mut self, geom: Geometry) {
        debug!(
            "Updating geometry for window {}: x: {}, y: {}, h: {}, w: {}",
            self.id, geom.x, geom.y, geom.height, geom.width
        );
        self.geom = geom;
    }

    /// Sets the width of the window.
    pub fn set_width(&mut self, x: i32) {
        self.geom.width = x;
    }

    /// Sets the height of the window.
    pub fn set_height(&mut self, y: i32) {
        self.geom.height = y;
    }

    /// Sets the x coordinate of the window.
    pub fn set_pos_x(&mut self, x: i32) {
        self.geom.x = x;
    }

    /// Sets the y coordinate of the window.
    pub fn set_pos_y(&mut self, y: i32) {
        self.geom.y = y;
    }

    /// Updates the width by a given delta.
    pub fn update_width(&mut self, dx: i32) {
        self.geom.width += dx;
    }
    /// Updates the height by given delta.
    pub fn update_height(&mut self, dy: i32) {
        self.geom.height += dy;
    }
    /// Updates the x coordinate of the window by a given delta.
    pub fn update_pos_x(&mut self, dx: i32) {
        self.geom.x += dx;
    }
    /// Updates the y coordinate of the window by a given delta.
    pub fn update_pos_y(&mut self, dy: i32) {
        self.geom.y += dy;
    }
}

/// Possible errors returned by the X connection.
#[non_exhaustive]
#[derive(Debug, Error, Clone)]
pub enum XError {
    /// An error when establishing a connection with the server.
    #[error("X connection error: {0}")]
    Connection(String),

    /// An error caused by a malformed protocol request.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// No screens were found by the window manager.
    #[error("Could not find screens from X server")]
    NoScreens,

    /// An invalid screen was selected.
    #[error("Unknown screen selected")]
    InvalidScreen,

    /// Some error caused by a malformed RandR request.
    #[error("RandR error: {0}")]
    RandrError(String),

    /// An internal server error.
    #[error("X server error: {0}")]
    ServerError(String),

    /// An error converting property or client message data.
    #[error("Error converting client message data")]
    ConversionError,

    /// A request for window properties returned malformed data.
    #[error("Invalid property data: {0}")]
    InvalidPropertyData(String),

    /// The request could not be fulfilled by the X server.
    #[error("Could not complete specified request: {0}")]
    RequestError(&'static str),

    /// Some error not tracked by ToaruWM.
    #[error("{0}")]
    OtherError(String),
}

/// Result type for XConn.
pub type Result<T> = ::core::result::Result<T, XError>;

/// A trait used to define the interface between toaruwm and the X server.
///
/// XConn provides an abstraction layer for talking to an underlying X server.
/// Its methods are designed to provide as thin a layer as possible,
/// often mapping directly to X server protocol requests, with type
/// conversion to present dependency-agnostic types.
///
/// An XConn implementation should also provide a way to manage X atoms.
/// Its `atom()` method should intern an Atom if not known, and
/// the implementation should store this in its internal state in some way.
/// While this functionality is not required, it is heavily encouraged.
///
/// An implementation of `XConn` is required for using a [WindowManager][1].
/// The backend library used does not directly appear inside `WindowManager`.
/// Thus, it is possible to create your own XConn type using a different
/// library, possibly using XLib, and in theory this crate can run on
/// any display server implementing the X protocol, given a proper
/// implementor of `XConn`.
///
/// This crate provides two implementations of XConn: [XCBConn][2] and
/// [X11RBConn][3].
///
/// [1]: crate::manager::WindowManager
/// [2]: crate::x::xcb::XCBConn
/// [3]: crate::x::x11rb::X11RBConn
pub trait XConn {
    //* General X server operations

    /// Receives the next event from the X server.
    ///
    /// If no events are queued, returns Ok(None),
    /// allowing the event loop to continue and handle other processing.
    /// If the connection has an error, it returns Err.
    ///
    /// Else, it returns Ok(Some(event)).
    fn poll_next_event(&self) -> Result<Option<XEvent>>;

    /// Returns the ID and geometry of the root window.
    fn get_root(&self) -> XWindow;

    /// Returns the geometry of a given window.
    fn get_geometry(&self, window: XWindowID) -> Result<Geometry>;

    /// Queries the given window and all its children.
    fn query_tree(&self, window: XWindowID) -> Result<Vec<XWindowID>>;

    /// Queries the X server for pointer data.
    fn query_pointer(&self, window: XWindowID) -> Result<PointerQueryReply>;

    /// Returns randr data on all connected screens.
    fn all_outputs(&self) -> Result<Vec<Screen>>;

    /// Get the value of an atom by its name.
    ///
    /// You can use [Atom][1]'s `as_ref()` method to get a
    /// known atom's string representation.
    ///
    /// If the atom is unknown, intern it.
    ///
    /// [1]: crate::x::atom::Atom
    fn atom(&self, atom: &str) -> Result<XAtom>;

    /// Looks up the name of an atom.
    fn lookup_atom(&self, atom: XAtom) -> Result<String>;

    /// Looks up the value of an interned atom given its name.
    ///
    /// If the atom is not interned, None should be returned.
    fn lookup_interned_atom(&self, name: &str) -> Option<XAtom>;

    /// Grabs the keyboard.
    fn grab_keyboard(&self) -> Result<()>;

    /// Ungrabs the keyboard.
    fn ungrab_keyboard(&self) -> Result<()>;

    /// Grabs a key-modmask combo for a given window.
    fn grab_key(&self, kb: Keybind, window: XWindowID) -> Result<()>;

    /// Ungrabs a key-modmask combo for a given window.
    fn ungrab_key(&self, kb: Keybind, window: XWindowID) -> Result<()>;

    /// Grabs a mouse button-mask combo for a given window.
    ///
    /// `confine` denotes whether or not the event should be generated.
    fn grab_button(&self, mb: Mousebind, window: XWindowID, confine: bool) -> Result<()>;

    /// Ungrabs a mouse button-mask combo for a given window.
    fn ungrab_button(&self, mb: Mousebind, window: XWindowID) -> Result<()>;

    /// Grabs the pointer.
    fn grab_pointer(&self, winid: XWindowID, mask: u32) -> Result<()>;

    /// Ungrabs the pointer.
    fn ungrab_pointer(&self) -> Result<()>;

    //* Window-related operations
    /// Create a new window.
    fn create_window(&self, ty: WindowClass, geom: Geometry, managed: bool) -> Result<XWindowID>;

    /// Maps a given window.
    fn map_window(&self, window: XWindowID) -> Result<()>;

    /// Unmaps a given window.
    fn unmap_window(&self, window: XWindowID) -> Result<()>;

    /// Destroys a window.
    ///
    /// Implementors should make use of the provided
    /// `XConn::win_supports()` method to delete the window
    /// via ICCCM WM_DELETE_WINDOW if supported.
    fn destroy_window(&self, window: XWindowID) -> Result<()>;

    /// Sends a message to a given client.
    fn send_client_message(&self, window: XWindowID, data: ClientMessageEvent) -> Result<()>;

    /// Sets the input focus to a given window.
    fn set_input_focus(&self, window: XWindowID) -> Result<()>;

    /// Set the geometry for a given window.
    fn set_geometry(&self, window: XWindowID, geom: Geometry) -> Result<()>;

    /// Set the property for a given window.
    fn set_property(&self, window: XWindowID, prop: &str, data: Property) -> Result<()>;

    /// Retrieves a given property for a given window by its atom name.
    fn get_property(&self, prop: &str, window: XWindowID) -> Result<Option<Property>>;

    /// Sets the root screen.
    fn set_root_scr(&mut self, scr: i32);

    /// Change window attributes for a given window.
    fn change_window_attributes(&self, window: XWindowID, attrs: &[ClientAttrs]) -> Result<()>;

    /// Configure a given window.
    fn configure_window(&self, window: XWindowID, attrs: &[ClientConfig]) -> Result<()>;

    /// Reparent a window under a given parent.
    fn reparent_window(&self, window: XWindowID, parent: XWindowID) -> Result<()>;
    //fn create_window(&self);

    //* provided methods to make my life easier
    // ICCCM-related operations
    /// Gets all ICCCM-defined client properties.
    fn get_client_properties(&self, window: XWindowID) -> XWinProperties {
        XWinProperties {
            wm_name: self.get_wm_name(window),
            wm_icon_name: self.get_wm_icon_name(window),
            wm_size_hints: self.get_wm_size_hints(window),
            wm_hints: self.get_wm_hints(window),
            wm_class: self.get_wm_class(window),
            wm_protocols: self.get_wm_protocols(window),
            wm_state: self.get_wm_state(window),
        }
    }

    /// Gets _NET_WM_NAME or failing which, WM_NAME.
    ///
    /// Returns an empty string in case of error or if neither is set.
    fn get_wm_name(&self, window: XWindowID) -> String {
        let prop = self.get_property(Atom::NetWmName.as_ref(), window);

        match prop {
            Ok(prop) => {
                if let Some(prop) = prop {
                    match prop {
                        Property::String(mut s) | Property::UTF8String(mut s) => {
                            return s.remove(0)
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => error!("{}", e),
        }

        let prop = self.get_property(Atom::WmName.as_ref(), window);

        match prop {
            Ok(prop) => {
                if let Some(prop) = prop {
                    match prop {
                        Property::String(mut s) | Property::UTF8String(mut s) => {
                            return s.remove(0)
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => error!("{}", e),
        }

        String::from("")
    }

    /// Gets WM_ICON_NAME.
    ///
    /// Returns an empty string in case of error.
    fn get_wm_icon_name(&self, window: XWindowID) -> String {
        let prop = self.get_property(Atom::WmIconName.as_ref(), window);

        match prop {
            Ok(prop) => {
                if let Some(prop) = prop {
                    match prop {
                        Property::String(mut s) | Property::UTF8String(mut s) => s.remove(0),
                        _ => "".into(),
                    }
                } else {
                    "".into()
                }
            }
            Err(_) => "".into(),
        }
    }

    /// Gets WM_NORMAL_HINTS.
    ///
    /// Returns None if not set or in case of error.
    fn get_wm_size_hints(&self, window: XWindowID) -> Option<WmSizeHints> {
        let prop = self
            .get_property(Atom::WmNormalHints.as_ref(), window)
            .ok()?;

        if let Some(Property::WMSizeHints(sh)) = prop {
            Some(sh)
        } else {
            debug!("Got wrong property: {:?}", prop);
            None
        }
    }

    /// Gets WM_HINTS.
    ///
    /// Returns None if not set or in case of error.
    fn get_wm_hints(&self, window: XWindowID) -> Option<WmHints> {
        let prop = self.get_property(Atom::WmHints.as_ref(), window).ok()?;

        if let Some(Property::WMHints(hints)) = prop {
            Some(hints)
        } else {
            debug!("Got wrong property: {:?}", prop);
            None
        }
    }

    /// Checks whether the the `WM_HINTS` property has the accepts-input
    /// flag set.
    fn accepts_input(&self, window: XWindowID) -> bool {
        if let Some(hints) = self.get_wm_hints(window) {
            hints.accepts_input
        } else {
            false
        }
    }

    /// Gets WM_CLASS.
    ///
    /// Returns a tuple of empty strings if not set or in case of error.
    fn get_wm_class(&self, window: XWindowID) -> (String, String) {
        use Property::{String, UTF8String};

        let prop = self
            .get_property(Atom::WmClass.as_ref(), window)
            .unwrap_or(None);

        match prop {
            Some(String(strs)) | Some(UTF8String(strs)) => (strs[0].to_owned(), strs[1].to_owned()),
            _ => {
                debug!(target: "get_wm_class", "expected strings, got: {:?}", prop);
                ("".into(), "".into())
            }
        }
    }

    /// Gets WM_PROTOCOLS.
    ///
    /// Returns None if not set or in case of error.
    fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<XAtom>> {
        let prop = self.get_property(Atom::WmProtocols.as_ref(), window).ok()?;

        if let Some(Property::Atom(atoms)) = prop {
            let mut ret = Vec::new();
            for atom in atoms {
                ret.push(self.atom(&atom).ok()?)
            }
            Some(ret)
        } else {
            None
        }
    }

    /// Check whether a window supports the given protocol.
    fn win_supports(&self, protocol: &str, id: XWindowID) -> bool {
        self.atom(protocol)
            .map(|atom| {
                self.get_wm_protocols(id)
                    .map(|protocols| protocols.contains(&atom))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    /// Gets ICCCM's `WM_STATE` hint.
    fn get_wm_state(&self, window: XWindowID) -> Option<WindowState> {
        let prop = self.get_property(Atom::WmState.as_ref(), window).ok()?;

        if let Some(Property::U32List(s, list)) = prop {
            if s != Atom::WmState.as_ref() {
                error!("Got wrong type for wm_state: {}", s);
                return None;
            }
            Some(match list[0] as i32 {
                1 => WindowState::Normal,
                3 => WindowState::Iconic,
                0 => WindowState::Withdrawn,
                n => {
                    error!("Expected 1, 3, or 0 for WM_STATE, got {}", n);
                    return None;
                }
            })
        } else {
            debug!(target: "get_wm_state", "window {} did not set WM_STATE", window);
            None
        }
    }

    /// Gets ICCCM's `WM_TRANSIENT_FOR` hint.
    fn get_wm_transient_for(&self, window: XWindowID) -> Option<XWindowID> {
        let prop = self
            .get_property(Atom::WmTransientFor.as_ref(), window)
            .ok()?;

        if let Some(Property::Window(ids)) = prop {
            if ids[0] == Xid(0) {
                warn!("Received window type but value is 0");
                None
            } else {
                Some(ids[0])
            }
        } else {
            debug!(
                target: "get_wm_transient_for",
                "window {} did not set WM_TRANSIENT_FOR", window
            );
            None
        }
    }

    /// Checks whether the `URGENCY` flag in `WM_HINTS` is set.
    fn get_urgency(&self, window: XWindowID) -> bool {
        if let Some(hints) = self.get_wm_hints(window) {
            hints.urgent()
        } else {
            false
        }
    }

    // EWMH-related operations
    /// Gets EWMH's `_NET_WM_WINDOW_TYPE`.
    fn get_window_type(&self, window: XWindowID) -> Result<Vec<String>> {
        let atom = Atom::NetWmWindowType.as_ref();

        match self.get_property(atom, window)? {
            Some(Property::Atom(atoms)) => Ok(atoms),
            None => Ok(vec![]),
            _ => Err(XError::InvalidPropertyData(
                "Expected Atom type for get_window_type".into(),
            )),
        }
    }

    /// Gets EWMH's `_NET_WM_STATE`.
    fn get_window_states(&self, window: XWindowID) -> Result<Vec<String>> {
        let atom = Atom::NetWmState.as_ref();

        match self.get_property(atom, window)? {
            Some(Property::Atom(atoms)) => Ok(atoms),
            None => Ok(vec![]),
            _ => Err(XError::InvalidPropertyData(
                "Expected Atom type for get_window_states".into(),
            )),
        }
    }

    /// Sets the _NET_SUPPORTED property on the root window.
    ///
    /// This indicates the protocols supported by the window manager.
    fn set_supported(&self, atoms: &[Atom]) -> Result<()> {
        self.set_property(
            self.get_root().id,
            Atom::NetSupported.as_ref(),
            Property::Atom(atoms.iter().map(|a| a.to_string()).collect()),
        )
    }

    /// Sets `_NET_WM_STATE` to the given atoms on the selected window.
    fn set_wm_state(&self, window: XWindowID, atoms: &[XAtom]) {
        let atoms = atoms
            .iter()
            .map(|s| self.lookup_atom(*s).unwrap_or_else(|_| String::new()))
            .filter(|s| !s.is_empty())
            .collect();
        self.set_property(window, Atom::NetWmState.as_ref(), Property::Atom(atoms))
            .unwrap_or_else(|_| error!("failed to set wm state"));
    }

    /// Returns whether a WindowManager should manage a window.
    fn should_manage(&self, window: XWindowID) -> bool {
        let win_type = match self.get_window_type(window) {
            Ok(atoms) => atoms
                .into_iter()
                .map(|s| Atom::from_str(&s))
                .filter_map(|a| a.ok())
                .collect::<Vec<Atom>>(),
            Err(_) => return true,
        };

        !UNMANAGED_WINDOW_TYPES.iter().any(|a| win_type.contains(a))
    }

    /// Returns whether a WindowManager should set a window to floating.
    ///
    /// Can accept user-specified classes that should float.
    fn should_float(&self, window: XWindowID, float_classes: &[String]) -> bool {
        let (_, class) = self.get_wm_class(window);

        if float_classes.iter().any(|s| *s == class) {
            return true;
        }

        let win_type = match self.get_window_type(window) {
            Ok(atoms) => atoms
                .into_iter()
                .map(|s| Atom::from_str(&s))
                .filter_map(|s| s.ok())
                .collect::<Vec<Atom>>(),
            Err(_) => return true,
        };

        AUTO_FLOAT_WINDOW_TYPES.iter().any(|a| win_type.contains(a))
    }
}

/// Abstracts over methods that all XConn implementations use internally.
pub(crate) trait XConnInner: XConn {}