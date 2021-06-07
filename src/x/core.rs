//! Core functionality of ToaruWM's interface with the X server.
//! 
//! This module defines core types and traits used throughout this
//! crate for directly interacting with the X server.

use thiserror::Error;

use crate::keybinds::{Keybind, Mousebind};

use crate::types::{
    Geometry,
    XWinProperties,
    Atom,
    NetWindowStates,
    ClientConfig,
    ClientAttrs,
};
use crate::core::{Screen, Client};
use super::{
    event::{XEvent, ClientMessageEvent},
    property::*,
};

/// An X server ID for a given window.
pub type XWindowID = u32;

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
    #[error("Could not establish a connection to the X server.")]
    Connection,

    /// An internal server error.
    #[error("X server error: {0}")]
    ServerError(String),

    /// An error converting property or client message data.
    #[error("Error converting client message data")]
    ConversionError,

    #[error("Invalid property data: {0}")]
    InvalidPropertyData(String),

    /// The request could not be fulfilled by the X server.
    #[error("Could not complete specified request.")]
    RequestError,

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
/// An implementation of `XConn` is required for using a [WindowManager][1].
/// The backend library used does not directly appear inside `WindowManager`.
/// Thus, it is possible to create your own XConn type using a different
/// library, posibly using XLib, and in theory this crate can run on
/// any display server implementing the X protocol, given a proper
/// implementor of `XConn`.
/// 
/// This crate provides an XCB-backed implementation of `XConn` - [XCBConn][2].
/// 
/// [1]: crate::manager::WindowManager
/// [2]: crate::x::xcb::XCBConn
#[allow(unused_variables)]
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

    /// Returns the ID of the root window.
    fn get_root(&self) -> XWindowID;

    /// Returns the geometry of a given window.
    fn get_geometry(&self, window: XWindowID) -> Result<Geometry>;

    /// Queries the given window and all its children.
    fn query_tree(&self, window: XWindowID) -> Result<Vec<XWindowID>>;

    /// Queries the X server for pointer data.
    fn query_pointer(&self, window: XWindowID) -> Result<PointerQueryReply>;

    /// Returns randr data on all connected screens.
    fn all_outputs(&self) -> Result<Vec<Screen>>;

    /// Get the value of an atom.
    /// 
    /// If the atom is unknown, intern it.
    fn atom(&self, atom: &str) -> Result<Atom>;

    /// Looks up the name of an atom.
    fn lookup_atom(&self, atom: Atom) -> Result<String>;

    /// Looks up the value of an interned atom given its name.
    /// 
    /// If the atom is not interned, None should be returned.
    fn lookup_interned_atom(&self, name: &str) -> Option<Atom>;

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

    /// Maps a given window.
    fn map_window(&self, window: XWindowID) -> Result<()>;

    /// Unmaps a given window.
    fn unmap_window(&self, window: XWindowID) -> Result<()>;

    /// Destroys a window.
    /// 
    /// Provides a reference to a Client so as to make use of ICCCM WM_DELETE_WINDOW.
    fn destroy_window(&self, window: &Client) -> Result<()>; 

    /// Sends a message to a given client.
    fn send_client_message(&self, window: XWindowID, data: ClientMessageEvent) -> Result<()>;

    /// Sets the input focus to a given window.
    fn set_input_focus(&self, window: XWindowID);

    /// Set the geometry for a given window.
    fn set_geometry(&self, window: XWindowID, geom: Geometry) -> Result<()>;

    /// Set the property for a given window.
    fn set_property(&self, window: XWindowID, prop: &str, data: Property) -> Result<()>;

    /// Retrieves a given property for a given window by its atom name.
    fn get_prop_str(&self, prop: &str, window: XWindowID) -> Result<Property>;

    /// Retrieves a given property for a given window by its atom value.
    fn get_prop_atom(&self, prop: Atom, window: XWindowID) -> Result<Property>;

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

    /// Gets WM_NAME.
    /// 
    /// Returns an empty string in case of error.
    fn get_wm_name(&self, window: XWindowID) -> String {
        let prop = self.get_prop_str("WM_NAME", window);

        match prop {
            Ok(prop) => {
                if let Property::UTF8String(mut prop) = prop {
                    prop.remove(0)
                } else { "".into() }
            }
            Err(_) => "".into() 
        }
    }

    /// Gets WM_ICON_NAME.
    /// 
    /// Returns an empty string in case of error.
    fn get_wm_icon_name(&self, window: XWindowID) -> String {
        let prop = self.get_prop_str("WM_ICON_NAME", window);

        match prop {
            Ok(prop) => {
                if let Property::UTF8String(mut prop) = prop {
                    prop.remove(0)
                } else { "".into() }
            }
            Err(_) => "".into() 
        }
    }

    /// Gets WM_NORMAL_HINTS.
    /// 
    /// Returns None if not set or in case of error.
    fn get_wm_size_hints(&self, window: XWindowID) -> Option<WmSizeHints> {
        let prop = self.get_prop_str("WM_NORMAL_HINTS", window).ok()?;

        if let Property::WMSizeHints(sh) = prop {
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
        let prop = self.get_prop_str("WM_HINTS", window).ok()?;

        if let Property::WMHints(hints) = prop {
            Some(hints)
        } else {
            debug!("Got wrong property: {:?}", prop);
            None
        }
    }

    /// Gets WM_CLASS.
    /// 
    /// Returns a tuple of empty strings if not set or in case of error.
    fn get_wm_class(&self, window: XWindowID) -> (String, String) {
        use Property::*;

        let prop = self.get_prop_str("WM_CLASS", window)
        .unwrap_or_else(|_| U8List(Vec::new()));

        match prop {
            String(strs) | UTF8String(strs) => {
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
    fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<Atom>> {
        let prop = self.get_prop_str("WM_PROTOCOLS", window).ok()?;

        if let Property::Atom(atoms) = prop {
            let mut ret = Vec::new();
            for atom in atoms {
                ret.push(self.atom(&atom).ok()?)
            }
            Some(ret)
        } else {
            None
        }
    }

    fn get_wm_state(&self, window: XWindowID) -> Option<WindowState> {
        let prop = self.get_prop_str("WM_STATE", window).ok()?;

        if let Property::U32List(list) = prop {
            Some(match list[0] as xcb_util::icccm::WmState {
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
        let prop = self.get_prop_str("WM_TRANSIENT_FOR", window).ok()?;

        if let Property::Window(ids) = prop {
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
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<String>> {
        let atom = self.atom(
            "_NET_WM_WINDOW_TYPE"
        ).expect("atom not interned");

        if let Ok(Property::Atom(atoms)) = self.get_prop_atom(atom, window) {
            Some(atoms)
        } else {
            error!("Expected Atom type for get_window_type");
            None
        }
    }

    fn get_window_states(&self, window: XWindowID) -> NetWindowStates {
        todo!()
    }

    fn set_supported(&self, screen_idx: i32, atoms: &[Atom]) {
        todo!()
    }

    fn set_wm_state(&self, window: XWindowID, atoms: &[Atom]) {
        todo!()
    }
}