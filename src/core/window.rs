//! This module exports Client and ClientRing, which encapsulate
//! data about windows and manage them internally respectively.

use std::collections::HashSet;

use super::{Ring, Selector};

use crate::x::{
    core::{XWindow, XWindowID, XConn, XAtom},
    property::{WindowState},
};
use crate::core::types::{
    Geometry,
    WinLayoutState,
    NetWindowStates,
    BorderStyle,
    ClientAttrs,
    ClientConfig,
};

/// A Ring of type Client.
///
/// Contains additional methods more specific to window management.
pub type ClientRing = Ring<Client>;

impl ClientRing {

    /// Wrapper around `Ring::remove` that takes a window ID instead of index.
    pub fn remove_by_id(&mut self, id: XWindowID) -> Option<Client> {
        if let Some(i) = self.get_idx(id) {
            self.remove(i)
        } else {
            None
        }
    }

    /// Wrapper around `Ring::index` that takes a window ID.
    pub fn get_idx(&self, id: XWindowID) -> Option<usize> {
        self.index(Selector::Condition(&|win| win.id() == id))
    }

    /// Returns a reference to the client containing the given window ID.
    pub fn lookup(&self, id: XWindowID) -> Option<&Client> {
        if let Some(i) = self.get_idx(id) {
            self.get(i)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the client containing the given ID.
    pub fn lookup_mut(&mut self, id: XWindowID) -> Option<&mut Client> {
        if let Some(i) = self.get_idx(id) {
            self.get_mut(i)
        } else {
            None
        }
    }

    /// Tests whether the Ring contains a client with the given ID.
    pub fn contains(&self, id: XWindowID) -> bool {
        matches!(self.element_by(|win| win.id() == id), Some(_))
    }

    /// Sets the focused element to the given client.
    pub fn set_focused_by_winid(&mut self, id: XWindowID) {
        if let Some(i) = self.get_idx(id) {
            self.focused = Some(i)
        } else {
            error!("Tried to focus a client not in the workspace")
        }
    }

    pub fn set_focused_by_idx(&mut self, idx: usize) {
        self.set_focused(idx);
    }

    /// Tests whether the client with the given ID is in focus.
    pub fn is_focused(&self, id: XWindowID) -> bool {
        if let Some(window) = self.focused() {
            window.id() == id
        } else {
            false
        }
    }
}

/// Represents an X server client.
/// It contains other data from the X server, stored locally,
/// such as ICCCM and EWMH properties.
/// 
/// Since this type is not Copy, it should not be passed around,
/// and should only be initialised and used within a `ClientRing`.
/// 
/// Instead of passing the entire Client around, XWindowIDs can
/// be used instead.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) xwindow: XWindow,
    name: String,
    icon_name: String,
    class: (String, String),

    initial_geom: Geometry,
    urgent: bool,
    transient_for: Option<XWindowID>,
    mapped_state: WindowState,
    net_states: NetWindowStates,
    layout_state: WinLayoutState,
    protocols: HashSet<XAtom>,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.xwindow.id == other.xwindow.id
    }
}

impl Client {

    /// Creates a new tiled Client.
    pub fn tiled<X: XConn>(from: XWindowID, conn: &X) -> Self {
        Self::new(from, conn, WinLayoutState::Tiled)
    }

    /// Creates a new floating Client.
    pub fn floating<X: XConn>(from: XWindowID, conn: &X) -> Self {
        Self::new(from, conn, WinLayoutState::Floating)
    }

    fn new<X: XConn>(from: XWindowID, conn: &X, layout: WinLayoutState) -> Self {
        let properties = conn.get_client_properties(from);
        Self {
            xwindow: XWindow::from(from),
            name: properties.wm_name().into(),
            icon_name: properties.wm_icon_name().into(),
            class: {
                let (class1, class2) = properties.wm_class();
                (class1.into(), class2.into())
            },
            initial_geom: Geometry::default(),
            transient_for: None,
            urgent: false,
            mapped_state: WindowState::Normal,
            net_states: NetWindowStates::new(),
            layout_state: layout,
            protocols: HashSet::new(),
        }
    }

    /// Returns the X ID of the client.
    #[inline(always)]
    pub fn id(&self) -> XWindowID {
        self.xwindow.id
    }

    /// Returns the x coordinate of the window.
    #[inline(always)]
    pub fn x(&self) -> i32 {
        self.xwindow.geom.x
    }

    /// Returns the y coordinate of the window.
    #[inline(always)]
    pub fn y(&self) -> i32 {
        self.xwindow.geom.y
    }

    /// Returns the height of the window.
    #[inline(always)]
    pub fn height(&self) -> u32 {
        self.xwindow.geom.height
    }

    /// Returns the width of the window.
    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.xwindow.geom.width
    }
    
    /// Returns the geometry of the window.
    #[inline(always)]
    pub fn geometry(&self) -> Geometry {
        self.xwindow.geom
    }

    /// Returns the value of WM_NAME.
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the value of WM_ICON_NAME.
    #[inline(always)]
    pub fn icon_name(&self) -> &str {
        &self.icon_name
    }

    /// Returns the value of WM_CLASS.
    #[inline(always)]
    pub fn class(&self) -> (&str, &str) {
        (&self.class.0, &self.class.1)
    }

    /// Tests whether the client is tiled.
    #[inline]
    pub fn is_tiled(&self) -> bool {
        if let WinLayoutState::Tiled = self.layout_state {
            return true
        }
        false
    }

    /// Tests whether the client is floating.
    #[inline]
    pub fn is_floating(&self) -> bool {
        if let WinLayoutState::Floating = self.layout_state {
            return true
        }
        false
    }

    /// Tests whether the client's urgent flag is set.
    #[inline(always)]
    pub fn is_urgent(&self) -> bool {
        self.urgent
    }

    /// Sets the client's state to tiled.
    /// 
    /// No-op if the client is already tiled.
    #[inline]
    pub fn set_tiled(&mut self) {
        self.layout_state = WinLayoutState::Tiled
    }

    /// Sets the client's state to floating.
    /// 
    /// No-op if the client is already floating.
    #[inline]
    pub fn set_floating(&mut self) {
        self.layout_state = WinLayoutState::Floating
    }

    /// Toggles the state of the client.
    #[inline]
    pub fn toggle_state(&mut self) {
        if let WinLayoutState::Floating = self.layout_state {
            debug!("Toggling window {} to tiled", self.id());
            self.layout_state = WinLayoutState::Tiled
        } else if let WinLayoutState::Tiled = self.layout_state {
            debug!("Toggling window {} to floating", self.id());
            self.layout_state = WinLayoutState::Floating
        }
    }

    /// Updates all the internal properties of the client.
    pub fn update_all_properties<X: XConn>(&mut self, conn: &X) {
        let properties = conn.get_client_properties(self.id());
        let initial_geom = if let Some(sizes) = properties.wm_size_hints() {
            debug!("Got size hints: {:#?}", sizes);
            Geometry {
                x: if let Some(pos) = sizes.position {pos.0} else {0},
                y: if let Some(pos) = sizes.position {pos.1} else {0},
                height: if let Some(dim) = sizes.size {dim.0 as u32} else {100},
                width: if let Some(dim) = sizes.size {dim.1 as u32} else {160},
            }
        } else {
            debug!("initial size is None");
            Geometry {
                x: 0,
                y: 0,
                height: 100,
                width: 160,
            }
        };
        self.name = properties.wm_name().into();
        self.icon_name = properties.wm_icon_name().into();

        if self.initial_geom == Geometry::zeroed() {
            self.initial_geom = initial_geom;
        }
        self.transient_for = conn.get_wm_transient_for(self.id());
        self.urgent = if let Some(hints) = properties.wm_hints() {
            hints.urgent()
        } else {false};
        self.mapped_state = if let Some(hints) = properties.wm_hints() {
            hints.initial_state
        } else {
            WindowState::Normal
        };
        self.net_states = match conn.get_window_states(self.id()) {
            Ok(atoms) => NetWindowStates::from_strings(atoms, conn),
            Err(e) => {
                warn!("Could not get _NET_WINDOW_STATE");
                error!("{}", e);
                NetWindowStates::new()
            }
        };
        if self.protocols.is_empty() {
            self.set_supported(conn);
        }
        if self.urgent {
            self.set_border(conn, BorderStyle::Urgent);
        }
        debug!("Updated properties: {:#?}", self);
    }

    /// Checks and updates the dynamic properties of the window.
    /// 
    /// Checked:
    /// 
    /// - WM_NAME
    /// - WM_ICON_NAME
    /// - WM_CLASS
    /// - WM_HINTS.Urgency
    pub fn update_dynamic<X: XConn>(&mut self, conn: &X) {
        self.name = conn.get_wm_name(self.id());
        self.icon_name = conn.get_wm_icon_name(self.id());
        self.class = conn.get_wm_class(self.id());
        self.urgent = conn.get_urgency(self.id());

        if self.urgent {
            self.set_border(conn, BorderStyle::Urgent);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_initial_geom(&mut self, geom: Geometry) {
        debug!("Setting initial geom to {:#?}", geom);
        self.initial_geom = geom;
    }

    /// Sets the border of the Client.
    /// 
    /// Should only be used internally.
    pub fn set_border<X: XConn>(&mut self, conn: &X, border: BorderStyle) {
        conn.change_window_attributes(
            self.id(), &[ClientAttrs::BorderColour(border)]
        ).unwrap_or_else(|e| error!("{}", e));
    }

    /// Maps the client.
    pub fn map<X: XConn>(&mut self, conn: &X) {
        //self.update_all_properties(conn);
        self.update_geometry(conn);
        conn.change_window_attributes(
            self.id(), 
            &[ClientAttrs::EnableClientEvents]
        ).unwrap_or_else(|e| error!("{}", e));
        conn.map_window(self.id()).unwrap_or_else(|e| error!("{}", e));
    }

    /// Unmaps the client.
    pub fn unmap<X: XConn>(&mut self, conn: &X) {
        self.mapped_state = WindowState::Iconic;
        conn.unmap_window(self.id()).unwrap_or_else(|e| error!("{}", e));
    }

    /// Sets the _NET_WM_STATES property.
    pub fn set_wm_states<X: XConn>(&self, conn: &X) {
        conn.set_wm_state(self.id(), &self.net_states);
    }

    /// Adds a new _NET_WM_STATES property.
    pub fn add_wm_state(&mut self, state: XAtom) {
        self.net_states.add(state)
    }

    /// Removes a _NET_WM_STATES property.
    pub fn remove_wm_state(&mut self, state: XAtom) {
        if self.net_states.contains(state) {
            self.net_states.remove(state);
        }
    }
    
    /// Configure the `Client` using a provided connection.
    /// 
    /// Use `Client::set_geometry` and `Client::update_geometry`
    /// to change client geometry instead of this method.
    pub fn configure<X: XConn>(&self, conn: &X, attrs: &[ClientConfig]) {
        conn.configure_window(self.id(), attrs).unwrap_or_else(|_|
            warn!("Could not configure window {}", self.id())
        );
    }

    /// Change client attributes.
    pub fn change_attributes<X: XConn>(&self, conn: &X, attrs: &[ClientAttrs]) {
        conn.change_window_attributes(self.id(), attrs).unwrap_or_else(|e| {
            error!("{}", e)
        })
    }

    /// Resize the window using _changes_ in height and width.
    /// 
    /// Does not do bounds checking.
    pub fn do_resize<X: XConn>(&mut self, conn: &X, dx: u32, dy: u32) {
        self.xwindow.update_height(dy);
        self.xwindow.update_width(dx);

        // let scrx = scr.xwindow.geom.x;
        // let scry = scr.xwindow.geom.y;
        // let scrh = scr.xwindow.geom.height;
        // let scrw = scr.xwindow.geom.width;

        // ensure_in_bounds(
        //     &mut self.xwindow.geom.height, 
        //     WIN_HEIGHT_MIN, scry + scrh - self.xwindow.geom.y);
        // ensure_in_bounds(&mut self.xwindow.geom.width, 
        //     WIN_WIDTH_MIN, scrx + scrw - self.xwindow.geom.x);

        conn.configure_window(self.xwindow.id,
            &[ClientConfig::Resize{h: self.height(), w: self.width()}]
        ).unwrap_or_else(|_|
            warn!("Could not configure window {}", self.id())
        );

        // debug!(
        //     "Updated geometry:\nx: {}, y: {}, h: {}, w: {}", 
        //     self.x(), self.y(), self.height(), self.width()
        // );
    }

    /// Move the window using _changes_ in window coordinates.
    /// 
    /// Does not do bounds checking.
    pub fn do_move<X: XConn>(&mut self, conn: &X, dx: i32, dy: i32) {
        self.xwindow.update_pos_y(dy);
        self.xwindow.update_pos_x(dx);

        // let scrx = scr.xwindow.geom.x;
        // let scry = scr.xwindow.geom.y;
        // let scrh = scr.xwindow.geom.height;
        // let scrw = scr.xwindow.geom.width;

        // ensure_in_bounds(&mut self.xwindow.geom.x, 
        //     scrx - self.xwindow.geom.width + MIN_ONSCREEN, 
        //     scrx + scrw - MIN_ONSCREEN);
        // ensure_in_bounds(&mut self.xwindow.geom.y, 
        //     scry - self.xwindow.geom.height + MIN_ONSCREEN, 
        //     scry + scrh - MIN_ONSCREEN);

        conn.configure_window(self.xwindow.id, 
            &[ClientConfig::Move{x: self.x(), y: self.y()}]
        ).unwrap_or_else(|_|
            warn!("Could not configure window {}", self.id())
        );

        // debug!(
        //     "Updated geometry:\nx: {}, y: {}, h: {}, w: {}", 
        //     self.x(), self.y(), self.height(), self.width()
        // );
    }

    /// Sets the geometry of the window, but does not update it to the X server.
    pub fn set_geometry(&mut self, geom: Geometry) {
        self.xwindow.set_geometry(geom);
    }

    /// Updates its geometry on the X server.
    /// 
    /// Normally called after `Client::set_geometry`.
    pub fn update_geometry<X: XConn>(&self, conn: &X) {
        conn.configure_window(self.xwindow.id,
            &[ClientConfig::Resize{h: self.height(), w: self.width()}]
        ).unwrap_or_else(|_|
            warn!("Could not configure window {}", self.id())
        );

        conn.configure_window(self.xwindow.id, 
            &[ClientConfig::Move{x: self.x(), y: self.y()}]
        ).unwrap_or_else(|_|
            warn!("Could not configure window {}", self.id())
        );
    }

    /// Updates and sets the Client geometry with a given Geometry.
    pub fn set_and_update_geometry<X: XConn>(&mut self, conn: &X, geom: Geometry) {
        self.set_geometry(geom);
        self.update_geometry(conn);
    }

    /// Sets the supported protocols for the client.
    pub fn set_supported<X: XConn>(&mut self, conn: &X) {
        if let Some(protocols) = conn.get_wm_protocols(self.id()) {
            for protocol in protocols {
                self.protocols.insert(protocol);
            }
        }
    }

    /// Tests whether the client supports this protocol.
    pub fn supports(&self, prtcl: XAtom) -> bool {
        self.protocols.contains(&prtcl)
    }
}