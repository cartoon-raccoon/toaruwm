//! Core functionality of ToaruWM's interface with the X server.
//! 
//! This module defines core types and traits used throughout this
//! crate for directly interacting with the X server.

use std::str::FromStr;

use thiserror::Error;

use crate::keybinds::{Keybind, Mousebind};

use crate::types::{
    Geometry,
    XWinProperties,
    ClientConfig,
    ClientAttrs,
};
use crate::core::{Screen};
use super::{
    event::{XEvent, ClientMessageEvent},
    property::*,
    atom::{
        Atom,
        UNMANAGED_WINDOW_TYPES,
        AUTO_FLOAT_WINDOW_TYPES,
    }
};

/// An X server ID for a given window.
pub type XWindowID = u32;

/// An X Atom, an unsigned 32-bit integer.
pub type XAtom = u32;

/// Contains the basic atoms and other constants used by 
/// the X specification protocol.
/// 
/// Re-exported from xcb-rs.
/// 
/// It does re-export some xcb-specific functions, but
/// most of the items used by ToaruWM are specific to the
/// X protocol, not the XCB library itself.
pub mod xproto {
    pub use xcb::xproto::*;
}

/// Window stacking modes defined by the X Protocol.
#[derive(Clone, Copy, Debug)]
pub enum StackMode {
    Above,
    Below,
    TopIf,
    BottomIf,
    Opposite,
}

/// Reply to a pointer query.
pub struct PointerQueryReply {
    pub same_screen: bool,
    pub root: XWindowID,
    pub child: XWindowID,
    pub root_x: i32,
    pub root_y: i32,
    pub win_x: i32,
    pub win_y: i32,
    pub mask: u16,
}

/// Representation of an X window with additional data (geometry).
#[derive(Debug, Clone, Copy)]
pub struct XWindow {
    pub id: XWindowID,
    pub geom: Geometry,
}

impl XWindow {
    pub fn zeroed() -> Self {
        XWindow {
            id: 0,
            geom: Geometry::zeroed(),
        }
    }

    pub fn with_data(id: XWindowID, geom: Geometry) -> Self {
        XWindow {
            id, geom,
        }
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
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowClass {
    /// An invisible window to make API calls
    CheckWin,
    /// A window that only accepts input
    InputOnly,
    /// A regular window. The Atom provided should be a valid
    /// _NET_WM_WINDOW_TYPE.
    InputOutput(Atom),
}

impl XWindow {
    /// Sets the geometry using an XConn object.
    pub fn set_geometry_conn<X: XConn>(&mut self, conn: &X) {
        match conn.get_geometry(self.id) {
            Ok(geom) => {
                debug!(
                    "Updating geometry (conn):\nx: {}, y: {}, h: {}, w: {}", 
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
    pub fn set_geometry(&mut self, geom: Geometry) {
        debug!(
            "Updating geometry for window {}:\nx: {}, y: {}, h: {}, w: {}", 
            self.id, geom.x, geom.y, geom.height, geom.width
        );
        self.geom = geom;
    }

    /// Sets the width of the window.
    pub fn set_width(&mut self, x: u32) {
        self.geom.width = x;
    }

    /// Sets the height of the window.
    pub fn set_height(&mut self, y: u32) {
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

    /// Updates the width by a given difference.
    pub fn update_width(&mut self, dx: u32) {
        self.geom.width += dx;
    }
    /// Updates the height by given difference.
    pub fn update_height(&mut self, dy: u32) {
        self.geom.height += dy;
    }
    /// Updates the x coordinate of the window by a given difference.
    pub fn update_pos_x(&mut self, dx: i32) {
        self.geom.x += dx;
    }
    /// Updates the y coordinate of the window by a given difference.
    pub fn update_pos_y(&mut self, dy: i32) {
        self.geom.y += dy;
    }
}

/// Possible errors returned by the X connection.
#[derive(Debug, Error, Clone)]
pub enum XError {
    /// An error when establishing a connection with the server.
    #[error("X connection error: {0}")]
    Connection(String),

    #[error("Could not find screens from X server")]
    NoScreens,

    #[error("Unknown screen selected")]
    InvalidScreen,

    #[error("RandR error: {0}")]
    RandrError(String),
    
    /// An internal server error.
    #[error("X server error: {0}")]
    ServerError(String),

    /// An error converting property or client message data.
    #[error("Error converting client message data")]
    ConversionError,

    #[error("Invalid property data: {0}")]
    InvalidPropertyData(String),

    /// The request could not be fulfilled by the X server.
    #[error("Could not complete specified request: {0}")]
    RequestError(&'static str),

    #[error("{0}")]
    OtherError(String)
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
/// 
/// An implementation of `XConn` is required for using a [WindowManager][1].
/// The backend library used does not directly appear inside `WindowManager`.
/// Thus, it is possible to create your own XConn type using a different
/// library, possibly using XLib, and in theory this crate can run on
/// any display server implementing the X protocol, given a proper
/// implementor of `XConn`.
/// 
/// This crate provides an XCB-backed implementation of `XConn` - [XCBConn][2].
/// 
/// [1]: crate::manager::WindowManager
/// [2]: crate::x::xcb::XCBConn
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

    /// Grabs a key-mask combo for a given window.
    fn grab_key(&self, kb: Keybind, window: XWindowID) -> Result<()>;

    /// Ungrabs a key-mask combo for a given window.
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
    fn set_input_focus(&self, window: XWindowID);

    /// Set the geometry for a given window.
    fn set_geometry(&self, window: XWindowID, geom: Geometry) -> Result<()>;

    /// Set the property for a given window.
    fn set_property(&self, window: XWindowID, prop: &str, data: Property) -> Result<()>;

    /// Retrieves a given property for a given window by its atom name.
    fn get_prop(&self, prop: &str, window: XWindowID) -> Result<Option<Property>>;

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
        let prop = self.get_prop(Atom::NetWmName.as_ref(), window);

        match prop {
            Ok(prop) => {
                if let Some(prop) = prop {
                    match prop {
                        Property::String(mut s) | 
                        Property::UTF8String(mut s) => {
                            return s.remove(0)
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => error!("{}", e),
        }

        let prop = self.get_prop(Atom::WmName.as_ref(), window);

        match prop {
            Ok(prop) => {
                if let Some(prop) = prop {
                    match prop {
                        Property::String(mut s) | 
                        Property::UTF8String(mut s) => {
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
        let prop = self.get_prop(Atom::WmIconName.as_ref(), window);

        match prop {
            Ok(prop) => {
                if let Some(prop) = prop {
                    match prop {
                        Property::String(mut s) |
                        Property::UTF8String(mut s) => {
                            s.remove(0)
                        }
                        _ => "".into()
                    }
                } else { "".into() }
            }
            Err(_) => "".into() 
        }
    }

    /// Gets WM_NORMAL_HINTS.
    /// 
    /// Returns None if not set or in case of error.
    fn get_wm_size_hints(&self, window: XWindowID) -> Option<WmSizeHints> {
        let prop = self.get_prop(Atom::WmNormalHints.as_ref(), window).ok()?;

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
        let prop = self.get_prop(Atom::WmHints.as_ref(), window).ok()?;

        if let Some(Property::WMHints(hints)) = prop {
            Some(hints)
        } else {
            debug!("Got wrong property: {:?}", prop);
            None
        }
    }

    fn accepts_input(&self, window: XWindowID) -> bool {
        if let Some(hints) = self.get_wm_hints(window) {
            hints.accepts_input
        } else {false}
    }

    /// Gets WM_CLASS.
    /// 
    /// Returns a tuple of empty strings if not set or in case of error.
    fn get_wm_class(&self, window: XWindowID) -> (String, String) {
        use Property::{String, UTF8String};

        let prop = self.get_prop(Atom::WmClass.as_ref(), window)
        .unwrap_or(None);

        match prop {
            Some(String(strs)) | Some(UTF8String(strs)) => {
                (strs[0].to_owned(), strs[1].to_owned())
            }
            _ => {
                debug!("Got wrong property: {:?}", prop);
                ("".into(), "".into())
            }
        }
    }

    /// Gets WM_PROTOCOLS.
    /// 
    /// Returns None if not set or in case of error.
    fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<XAtom>> {
        let prop = self.get_prop(Atom::WmProtocols.as_ref(), window).ok()?;

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
        self.atom(protocol).map(|atom| {
            self.get_wm_protocols(id).map(|protocols| {
                protocols.contains(&atom)
            }).unwrap_or(false)
        }).unwrap_or(false)
    }

    fn get_wm_state(&self, window: XWindowID) -> Option<WindowState> {
        let prop = self.get_prop(Atom::WmState.as_ref(), window).ok()?;

        if let Some(Property::U32List(s, list)) = prop {
            if s != Atom::WmState.as_ref() {
                error!("Got wrong type for wm_state: {}", s);
                return None
            }
            Some(match list[0] as i32 {
                1 => WindowState::Normal,
                3 => WindowState::Iconic,
                0 => WindowState::Withdrawn,
                n => {
                    error!("Expected 1, 3, or 0 for WM_STATE, got {}", n);
                    return None
                }
            })
        } else {
            error!("Expected Property::U32List, got {:?}", prop);
            None
        }
    }

    fn get_wm_transient_for(&self, window: XWindowID) -> Option<XWindowID> {
        let prop = self.get_prop(Atom::WmTransientFor.as_ref(), window).ok()?;

        if let Some(Property::Window(ids)) = prop {
            if ids[0] == 0 {
                warn!("Received window type but value is 0");
                None
            } else {Some(ids[0])}
        } else {
            error!("Expected window type, got {:?}", prop);
            None
        }
    }

    fn get_urgency(&self, window: XWindowID) -> bool {
        if let Some(hints) = self.get_wm_hints(window) {
            hints.urgent()
        } else {false}
    }

    // EWMH-related operations
    fn get_window_type(&self, window: XWindowID) -> Result<Vec<String>> {
        let atom = Atom::NetWmWindowType.as_ref();

        if let Some(Property::Atom(atoms)) = self.get_prop(atom, window)? {
            Ok(atoms)
        } else {
            Err(XError::InvalidPropertyData(
                "Expected Atom type for get_window_type".into()
            ))
        }
    }

    fn get_window_states(&self, window: XWindowID) -> Result<Vec<String>> {
        let atom = Atom::NetWmState.as_ref();

        if let Some(Property::Atom(atoms)) = self.get_prop(atom, window)? {
            Ok(atoms)
        } else {
            Err(XError::InvalidPropertyData(
                "Expected Atom type for get_window_states".into()
            ))
        }
        
    }

    /// Sets the _NET_SUPPORTED property on the root window.
    /// 
    /// This indicated the protocols supported by the window manager.
    fn set_supported(&self, atoms: &[Atom]) -> Result<()> {
        self.set_property(
            self.get_root().id,
            Atom::NetSupported.as_ref(),
            Property::Atom(atoms.iter()
                .map(|a| a.to_string())
                .collect()
            )
        )
    }

    fn set_wm_state(&self, window: XWindowID, atoms: &[XAtom]) {
        let atoms = atoms.iter()
            .map(|s| self.lookup_atom(*s).unwrap_or_else(|_|String::new()))
            .filter(|s| !s.is_empty())
            .collect();
        self.set_property(
            window, 
            Atom::NetWmState.as_ref(), 
            Property::Atom(atoms)
        ).unwrap_or_else(|_| error!("failed to set wm state"));
    }

    /// Returns whether a WindowManager should manage a window.
    fn should_manage(&self, window: XWindowID) -> bool {
        let win_type = match self.get_window_type(window) {
            Ok(atoms) => atoms
                .into_iter()
                .map(|s| Atom::from_str(&s))
                .filter(|s| s.is_ok())
                .map(|a| a.unwrap())
                .collect::<Vec<Atom>>(),
            Err(_) => return true
        };
    
        !UNMANAGED_WINDOW_TYPES.iter().any(|a| win_type.contains(a))
    }

    /// Returns whether a WindowManager should set a window to floating.
    /// 
    /// Can accept user-specified classes that should float.
    fn should_float(&self, window: XWindowID, float_classes: &[String]) -> bool {
        let (_, class) = self.get_wm_class(window);

        if float_classes.iter().any(|s| *s == class) {
            return true
        }

        let win_type = match self.get_window_type(window) {
            Ok(atoms) => atoms
                .into_iter()
                .map(|s| Atom::from_str(&s))
                .filter(|s| s.is_ok())
                .map(|a| a.unwrap())
                .collect::<Vec<Atom>>(),
            Err(_) => return true
        };

        AUTO_FLOAT_WINDOW_TYPES.iter().any(|a| win_type.contains(a))
    }
}