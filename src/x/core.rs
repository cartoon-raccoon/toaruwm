use thiserror::Error;

use crate::keybinds::{Keybind, Mousebind};

use crate::types::{
    Geometry,
    XWinProperties,
    WindowState,
    Atom,
    NetWindowStates,
};
use crate::core::{Screen, Client};
use super::event::{XEvent, ClientMessageEvent};

pub type XWindowID = u32;

/// Contains the basic atoms and other constants used by 
/// the X specification protocol.
/// 
/// Re-exported from xcb-rs.
/// 
/// It does re-export some xcb-specific functions, but
/// most of the items used by ToaruWM are specific to the
/// X protocol, not the xcb library itself.
pub mod xproto {
    pub use xcb::xproto::*;
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

/// X server properties.
#[derive(Debug, Clone)]
pub enum Property {
    Atom(Vec<String>),
    Bytes(Vec<u32>),
    Cardinal(u32),
    UTF8String(Vec<String>),
    Window(Vec<XWindowID>),
    WMHints(WmHints),
    WMSizeHints(SizeHints),
}

/// ICCCM-defined window hints (WM_HINTS).
#[derive(Debug, Clone, Copy)]
pub struct WmHints {
    pub state: WindowState,
    pub urgent: bool,
    //todo: add pixmaps
}

/// ICCCM-defined window size hints (WM_SIZE_HINTS).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SizeHints {
    pub position: Option<(i32, i32)>,
    pub size: Option<(i32, i32)>,
    pub min_size: Option<(i32, i32)>,
    pub max_size: Option<(i32, i32)>,
    pub resize: Option<(i32, i32)>,
    pub min_aspect: Option<(i32, i32)>,
    pub max_aspect: Option<(i32, i32)>,
    pub base: Option<(i32, i32)>,
    pub gravity: Option<u32>
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
    #[error("Could not establish a connection to the X server.")]
    Connection,
    #[error("X server error: {0}")]
    ServerError(String),
    #[error("Error converting client message data")]
    ConversionError,
    #[error("Could not complete specified request.")]
    RequestError,
}

/// Result type for XConn.
pub type Result<T> = ::core::result::Result<T, XError>;

use xproto::EventMask;

pub trait XConn {
    // General X server operations
    fn get_next_event(&self) -> XEvent;
    fn get_root(&self) -> XWindowID;
    fn get_geometry(&self, window: XWindowID) -> Result<Geometry>;
    fn query_tree(&self) -> Vec<XWindowID>;
    fn query_pointer(&self, window: XWindowID) -> Result<PointerQueryReply>;
    fn all_outputs(&self) -> Vec<Screen>;
    fn intern_atom(&self, atom: &str) -> Result<Atom>;
    fn lookup_atom(&self, atom: Atom) -> Result<String>;
    fn lookup_interned_atom(&self, atom: Atom) -> Option<&str>;
    fn lookup_interned_atom_name(&self, name: &str) -> Option<Atom>;
    fn grab_keyboard(&self) -> Result<()>;
    fn ungrab_keyboard(&self) -> Result<()>;
    fn grab_key(&self, kb: Keybind) -> Result<()>;
    fn ungrab_key(&self, kb: Keybind) -> Result<()>;
    fn grab_button(&self, mb: Mousebind) -> Result<()>;
    fn ungrab_button(&self, mb: Mousebind) -> Result<()>;
    fn grab_pointer(&self, winid: XWindowID, mask: EventMask);
    fn ungrab_pointer(&self);

    // Window-related operations
    fn map_window(&self, window: XWindowID);
    fn unmap_window(&self, window: XWindowID);
    //Provides a reference to a Client so as to make use of ICCCM WM_DESTROY_WINDOW
    fn destroy_window(&self, window: &Client); 
    fn send_client_message(&self, window: XWindowID, data: ClientMessageEvent);
    fn set_input_focus(&self, window: XWindowID);
    fn set_geometry(&self, window: XWindowID, geom: Geometry);
    fn set_property(&self, window: XWindowID);
    fn get_prop_str(&self, prop: &str, window: XWindowID) -> Result<Property>;
    fn get_prop_atom(&self, prop: Atom, window: XWindowID) -> Result<Property>;
    fn set_root_scr(&mut self, scr: i32);
    fn change_window_attributes(&self, window: XWindowID, attrs: &[(u32, u32)]) -> Result<()>;
    fn configure_window(&self, window: XWindowID, attrs: &[(u16, u32)]) -> Result<()>;
    fn reparent_window(&self, window: XWindowID, parent: XWindowID) -> Result<()>;
    //fn create_window(&self);

    //* provided methods to make my life easier
    // ICCCM-related operations
    fn get_client_properties(&self, window: XWindowID) -> XWinProperties {
        todo!()
    }
    fn get_wm_name(&self, window: XWindowID) -> String {
        let prop = self.get_prop_atom(xproto::ATOM_WM_NAME, window);
        if prop.is_err() { 
            "".into() 
        } else {
            let prop = prop.unwrap();
            if let Property::UTF8String(prop) = prop {
                prop.remove(0)
            } else { "".into() }
        }
    }
    fn get_wm_icon_name(&self, window: XWindowID) -> String {
        todo!()
    }
    fn get_wm_size_hints(&self, window: XWindowID) -> Option<SizeHints> {
        todo!()
    }
    fn get_wm_hints(&self, window: XWindowID) -> Option<WmHints> {
        todo!()
    }
    fn get_wm_class(&self, window: XWindowID) -> Option<(String, String)> {
        todo!()
    }
    fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<Atom>> {
        todo!()
    }
    fn get_wm_state(&self, window: XWindowID) -> WindowState {
        todo!()
    }
    fn get_wm_transient_for(&self, window: XWindowID) -> Option<XWindowID> {
        todo!()
    }
    fn get_urgency(&self, window: XWindowID) -> bool {
        if let Some(hints) = self.get_wm_hints(window) {
            hints.urgent
        } else {false}
    }

    // EWMH-related operations
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<String>> {
        let atom = self.lookup_interned_atom_name(
            "_NET_WM_WINDOW_TYPE"
        ).expect("atom not interned");

        if let Some(Property::Atom(atoms)) = self.get_prop_atom(atom, window).ok() {
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