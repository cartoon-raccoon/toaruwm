//! Types used to represent and manage individual windows.
//!
//! This core of this module is the `Client` type, which represents
//! an individual window on the X server that is also managed
//! by a `WindowManager`.
//!
//! See the [`Client`] documentation for more details.

use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use tracing::instrument;
use tracing::{debug, error, warn};

use super::{ring::InsertPoint, Ring, Selector};

use crate::core::types::{
    BorderStyle, ClientAttrs, ClientConfig, Color, Geometry, NetWindowStates,
};
use crate::manager::RuntimeConfig;
use crate::x::{
    core::{XAtom, XConn, XWindow, XWindowID},
    property::WindowState,
};

/// A ring of Clients.
///
/// Contains additional methods more specific to window management.
///
/// It implements `Deref` and `DerefMut` to `Ring`, so you can
/// use all `Ring` methods on it.
///
/// The focused element of this ring is the window that currently
/// has the input focus.
///
/// A `ClientRing` also plays an important role in enforcing window
/// stacking, keeping all off-layout clients on top.
#[derive(Debug, Clone)]
pub struct ClientRing(Ring<Client>);
/* we still need to change focus on this everytime so we know
which window to cycle focus to */

impl Deref for ClientRing {
    type Target = Ring<Client>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ClientRing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for ClientRing {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientRing {
    /// Creates a new ClientRing.
    pub fn new() -> Self {
        Self(Ring::new())
    }

    /// Adds the Client at a given index.
    pub fn add_at_index(&mut self, idx: usize, win: Client) {
        self.insert(InsertPoint::Index(idx), win);
    }

    /// Wrapper around `Ring::remove` that takes a window ID instead of index.
    pub fn remove_by_id(&mut self, id: XWindowID) -> Option<Client> {
        let Some(i) = self.get_idx(id) else {
            return None
        };

        self.remove(i)
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
        self.get_idx(id).and_then(|i| self.get_mut(i))
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

    /// Sets the focused element by its index in the underlying Ring.
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
///
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
    fullscreen: bool,

    /* indicates whether a client count as part of the current layout */
    inside_layout: bool,
    transient_for: Option<XWindowID>,
    mapped_state: WindowState,
    net_states: NetWindowStates,
    protocols: HashSet<XAtom>,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.xwindow.id == other.xwindow.id
    }
}

impl Client {
    /// Creates a new Client.
    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(conn)))]
    pub fn new<X: XConn>(from: XWindowID, conn: &X) -> Self {
        let properties = conn.get_client_properties(from);
        Self {
            xwindow: XWindow::from(from),
            name: properties.wm_name().into(),
            icon_name: properties.wm_icon_name().into(),
            class: {
                let (class1, class2) = properties.wm_class();
                (class1.into(), class2.into())
            },
            initial_geom: if let Ok(geom) = conn.get_geometry(from) {
                geom
            } else {
                Geometry::default()
            },
            transient_for: conn.get_wm_transient_for(from),
            urgent: false,
            fullscreen: false,
            inside_layout: true,
            mapped_state: WindowState::Normal,
            net_states: NetWindowStates::new(),
            protocols: HashSet::new(),
        }
    }

    /// Returns a Client that should float.
    pub fn outside_layout<X: XConn>(from: XWindowID, conn: &X) -> Self {
        let mut new = Self::new(from, conn);
        new.inside_layout = false;

        new
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
    pub fn height(&self) -> i32 {
        self.xwindow.geom.height
    }

    /// Returns the width of the window.
    #[inline(always)]
    pub fn width(&self) -> i32 {
        self.xwindow.geom.width
    }

    /// Returns the geometry of the window.
    #[inline(always)]
    pub fn geometry(&self) -> Geometry {
        self.xwindow.geom
    }

    /// Returns the initial geometry of the window, as set by the
    /// program that created it.
    #[inline(always)]
    pub fn initial_geom(&self) -> Geometry {
        self.initial_geom
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

    /// Tests whether the client's urgent flag is set.
    #[inline(always)]
    pub fn is_urgent(&self) -> bool {
        self.urgent
    }

    /// Returns whether the Client is fullscreen.
    ///
    /// Note that this is not the actual state of the client on
    /// the X server, this is the state as tracked by ToaruWM.
    #[inline(always)]
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Returns whether the Client should be floated regardless
    /// of the current layout.
    #[inline(always)]
    pub fn is_off_layout(&self) -> bool {
        !self.inside_layout
    }

    /// Mark a Client as outside of the layout.
    ///
    /// # Note
    ///
    /// This only changes the internal state of the window.
    /// It is the caller's responsibility to effect
    /// any server-side implications of this change based on
    /// their policy towards windows on/off layout.
    pub fn set_off_layout(&mut self) {
        self.inside_layout = false;
    }

    /// Mark a Client as inside of the layout.
    ///
    /// # Note
    ///
    /// This only changes the internal state of the window.
    /// It is the caller's responsibility to effect
    /// any server-side implications of this change based on
    /// their policy towards windows on/off layout.
    pub fn set_on_layout(&mut self) {
        self.inside_layout = true;
    }

    /// Updates all the internal properties of the client.
    #[cfg_attr(debug_assertions, instrument(level = "debug", skip_all))]
    pub fn update_all_properties<X, C>(&mut self, conn: &X, cfg: &C)
    where
        X: XConn,
        C: RuntimeConfig,
    {
        let properties = conn.get_client_properties(self.id());
        let initial_geom = if let Some(sizes) = properties.wm_size_hints() {
            debug!("Got size hints: {:#?}", sizes);
            Geometry {
                x: if let Some(pos) = sizes.position {
                    pos.0
                } else {
                    0
                },
                y: if let Some(pos) = sizes.position {
                    pos.1
                } else {
                    0
                },
                height: if let Some(dim) = sizes.size {
                    dim.0
                } else {
                    100
                },
                width: if let Some(dim) = sizes.size {
                    dim.1
                } else {
                    160
                },
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
        } else {
            false
        };
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
            self.set_border(conn, cfg.border_style(BorderStyle::Urgent));
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
    pub fn update_dynamic<X, C>(&mut self, conn: &X, cfg: &C)
    where
        X: XConn,
        C: RuntimeConfig,
    {
        self.name = conn.get_wm_name(self.id());
        self.icon_name = conn.get_wm_icon_name(self.id());
        self.class = conn.get_wm_class(self.id());
        self.urgent = conn.get_urgency(self.id());

        if self.urgent {
            self.set_border(conn, cfg.border_style(BorderStyle::Urgent));
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
    pub fn set_border<X: XConn>(&mut self, conn: &X, border: Color) {
        conn.change_window_attributes(self.id(), &[ClientAttrs::BorderColour(border)])
            .unwrap_or_else(|e| error!("{}", e));
    }

    /// Maps the client.
    pub fn map<X: XConn>(&mut self, conn: &X) {
        trace!("mapping window {}", self.xwindow.id);
        // note that we do not update our geometry here.
        // all geometry updates are done by the layout engine.
        conn.change_window_attributes(self.id(), &[ClientAttrs::EnableClientEvents])
            .unwrap_or_else(|e| error!("{}", e));
        conn.map_window(self.id())
            .unwrap_or_else(|e| error!("{}", e));
        trace!("mapping complete");
    }

    /// Unmaps the client.
    pub fn unmap<X: XConn>(&mut self, conn: &X) {
        self.mapped_state = WindowState::Iconic;
        conn.unmap_window(self.id())
            .unwrap_or_else(|e| error!("{}", e));
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
        trace!(
            "configuring window {} with attributes {:?}",
            self.xwindow.id,
            attrs
        );
        conn.configure_window(self.id(), attrs)
            .unwrap_or_else(|e| warn!("Could not configure window {}, got error {}", self.id(), e));
    }

    /// Change client attributes.
    pub fn change_attributes<X: XConn>(&self, conn: &X, attrs: &[ClientAttrs]) {
        trace!(
            "changing window {} attributes with {:?}",
            self.xwindow.id,
            attrs
        );
        conn.change_window_attributes(self.id(), attrs)
            .unwrap_or_else(|e| error!("{}", e))
    }

    /// Resize the window using _changes_ in height and width.
    ///
    /// Does not do bounds checking.
    pub fn do_resize<X: XConn>(&mut self, conn: &X, dx: i32, dy: i32) {
        self.xwindow.update_height(dy);
        self.xwindow.update_width(dx);

        conn.configure_window(
            self.xwindow.id,
            &[ClientConfig::Resize {
                h: self.height(),
                w: self.width(),
            }],
        )
        .unwrap_or_else(|e| warn!("Could not configure window {}, got error {}", self.id(), e));

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

        conn.configure_window(
            self.xwindow.id,
            &[ClientConfig::Move {
                x: self.x(),
                y: self.y(),
            }],
        )
        .unwrap_or_else(|e| warn!("Could not configure window {} with error {}", self.id(), e));

        // debug!(
        //     "Updated geometry:\nx: {}, y: {}, h: {}, w: {}",
        //     self.x(), self.y(), self.height(), self.width()
        // );
    }

    /// Sets the position of the window on the root window with respect to
    /// its gravity.
    pub fn set_position<X: XConn>(&mut self, conn: &X, x: i32, y: i32) {
        self.xwindow.set_pos_x(x);
        self.xwindow.set_pos_y(y);

        conn.configure_window(
            self.xwindow.id,
            &[ClientConfig::Move {
                x: self.x(),
                y: self.y(),
            }],
        )
        .unwrap_or_else(|e| warn!("Could not configure window {} with error {}", self.id(), e));
    }

    /// Sets the size of the window.
    pub fn set_size<X: XConn>(&mut self, conn: &X, height: i32, width: i32) {
        self.xwindow.set_height(height);
        self.xwindow.set_width(width);

        conn.configure_window(
            self.xwindow.id,
            &[ClientConfig::Resize {
                h: self.height(),
                w: self.width(),
            }],
        )
        .unwrap_or_else(|e| warn!("Could not configure window {} with error {}", self.id(), e));
    }

    /// Sets the geometry of the window, but does not update it to the X server.
    pub fn set_geometry(&mut self, geom: Geometry) {
        self.xwindow.set_geometry(geom);
    }

    /// Updates its geometry on the X server.
    ///
    /// Normally called after `Client::set_geometry`.
    pub fn update_geometry<X: XConn>(&self, conn: &X) {
        conn.configure_window(
            self.xwindow.id,
            &[ClientConfig::Resize {
                h: self.height(),
                w: self.width(),
            }],
        )
        .unwrap_or_else(|e| warn!("Could not configure window {} with error {}", self.id(), e));

        conn.configure_window(
            self.xwindow.id,
            &[ClientConfig::Move {
                x: self.x(),
                y: self.y(),
            }],
        )
        .unwrap_or_else(|e| warn!("Could not configure window {} with error {}", self.id(), e));
    }

    /// Updates and sets the Client geometry with a given Geometry.
    pub fn set_and_update_geometry<X: XConn>(&mut self, conn: &X, geom: Geometry) {
        self.set_geometry(geom);
        self.update_geometry(conn);
    }

    /// Sets the supported protocols for the client.
    pub fn set_supported<X: XConn>(&mut self, conn: &X) {
        trace!("setting supported protocols for window {}", self.xwindow.id);
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

/// Maintains the focusing order of the windows of screen.
#[derive(Debug, Clone)]
pub(crate) struct FocusStack(Ring<XWindowID>);

impl Deref for FocusStack {
    type Target = Ring<XWindowID>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FocusStack {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

#[allow(dead_code)]
impl FocusStack {
    /// Creates a new FocusStack.
    pub fn new() -> Self {
        Self(Ring::new())
    }

    pub fn add_by_layout_status(&mut self, id: XWindowID, clients: &ClientRing) {
        let Some(cl) = clients.lookup(id) else {
            warn!("could not find client with id {} in clientring", id);
            return
        };

        if cl.is_off_layout() {
            self.push(id);
        } else {
            let idx = self.partition_idx(clients);
            self.insert(InsertPoint::Index(idx), id);
        }
    }

    pub fn set_focused_by_winid(&mut self, id: XWindowID) {
        if let Some(idx) = self.get_idx(id) {
            self.set_focused(idx);
        } else {
            warn!("No window with id {} found", id)
        }
    }

    pub fn remove_by_id(&mut self, id: XWindowID) -> Option<XWindowID> {
        self.get_idx(id).and_then(|idx| self.remove(idx))
    }

    pub fn on_layout<'ws>(&'ws self, cl: &'ws ClientRing) -> impl Iterator<Item = &'ws XWindowID> {
        self.iter().filter(|id| {
            !(cl.lookup(**id)
                .expect("could not find client")
                .is_off_layout())
        })
    }

    pub fn off_layout<'ws>(&'ws self, cl: &'ws ClientRing) -> impl Iterator<Item = &'ws XWindowID> {
        self.iter().filter(|id| {
            cl.lookup(**id)
                .expect("could not find client")
                .is_off_layout()
        })
    }

    /// Moves the window with ID `id` to the top of its respective
    /// stack.
    ///
    /// If the window is off layout, it is moved to the front of
    /// the queue; if it is on layout, it is moved to the first
    /// index of the stacked windows.
    pub fn bubble_to_top(&mut self, id: XWindowID, c: &ClientRing) {
        if self.is_empty() {
            return;
        }
        let Some(idx) = c.get_idx(id) else {
            warn!("could not find window with ID {} in clientring", id);
            return
        };
        let Some(cl) = c.lookup(id) else {
            warn!("could not find window with ID {} in clientring", id);
            return
        };

        if cl.is_off_layout() {
            self.move_front(idx);
        } else {
            let n_idx = self.partition_idx(c);
            debug!("get partition idx {}, len {}", n_idx, self.len());
            self.move_to(idx, n_idx);
        }
    }

    /// Wrapper around `Ring::index` that takes a window ID.
    pub fn get_idx(&self, id: XWindowID) -> Option<usize> {
        self.0.index(Selector::Condition(&|win| *win == id))
    }

    /// Gets the index where the first window on layout resides.
    ///
    /// Assumes the `ClientRing` is indeed partitioned.
    //* precondition: the ring is already partitioned correctly */
    pub fn partition_idx(&self, clients: &ClientRing) -> usize {
        self.0
            .items
            .partition_point(|c| clients.lookup(*c).expect("no client found").is_off_layout())
    }
}
