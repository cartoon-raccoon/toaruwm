use thiserror::Error;

use crate::types::{
    Geometry,
    XWinProperties,
    WindowState,
    Atom,
    NetWindowStates,
};
use crate::core::Screen;
use super::event::XEvent;

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

/// ICCCM-defined window hints.
#[derive(Debug, Clone, Copy)]
pub struct WmHints {
    pub state: WindowState,
    pub urgent: bool,
    //todo: add pixmaps
}

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

#[derive(Debug, Error, Clone, Copy)]
pub enum XError {
    #[error("Could not establish a connection to the X server.")]
    Connection,
    #[error("Could not complete specified request.")]
    RequestError,
}

pub type Result<T> = ::core::result::Result<T, XError>;

pub trait XConn {
    // General X server operations
    fn get_next_event(&self) -> XEvent;
    fn get_root(&self) -> XWindowID;
    fn get_geometry(&self, window: XWindowID) -> Result<Geometry>;
    fn query_tree(&self) -> Vec<XWindowID>;
    fn query_pointer(&self, window: XWindowID) -> Result<PointerQueryReply>;
    fn all_outputs(&self) -> Vec<Screen>;
    fn get_prop(&self, prop: &str) -> Result<Property>;
    fn intern_atom(&self, atom: &str) -> Result<Atom>;

    // Window-related operations
    fn map_window(&self, window: XWindowID);
    fn unmap_window(&self, window: XWindowID);
    fn destroy_window(&self, window: XWindowID); //* don't forget to use icccm DESTROY_WINDOW
    fn set_input_focus(&self, window: XWindowID);
    fn set_geometry(&self, window: XWindowID, geom: Geometry);
    fn set_property(&self, window: XWindowID);
    fn set_root_scr(&mut self, scr: i32);
    fn change_window_attributes(&self, window: XWindowID, attrs: &[(u32, u32)]) -> Result<()>;
    fn configure_window(&self, window: XWindowID, attrs: &[(u16, u32)]);
    fn reparent_window(&self, window: XWindowID, parent: XWindowID);
    //fn create_window(&self);

    // ICCCM-related operations
    fn get_client_properties(&self, window: XWindowID) -> XWinProperties;
    fn get_wm_name(&self, window: XWindowID) -> String;
    fn get_wm_icon_name(&self, window: XWindowID) -> String;
    fn get_wm_size_hints(&self, window: XWindowID) -> Option<SizeHints>;
    fn get_wm_hints(&self, window: XWindowID) -> Option<WmHints>;    
    fn get_wm_class(&self, window: XWindowID) -> Option<(String, String)>;
    fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<Atom>>;
    fn get_wm_state(&self, window: XWindowID) -> WindowState;
    fn get_wm_transient_for(&self, window: XWindowID) -> Option<XWindowID>;
    fn get_urgency(&self, window: XWindowID) -> bool;

    // EWMH-related operations
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<Atom>>;
    fn get_window_states(&self, window: XWindowID) -> NetWindowStates;
    fn set_supported(&self, screen_idx: i32, atoms: &[Atom]);
    fn set_wm_state(&self, window: XWindowID, atoms: &[Atom]);
}