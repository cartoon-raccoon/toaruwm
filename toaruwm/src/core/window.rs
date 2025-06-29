//! Types used to represent and manage individual windows.
//!
//! This core of this module is the `Client` type, which represents
//! an individual window on the X server that is also managed
//! by a `WindowManager`.
//!
//! See the [`Window`] documentation for more details.

use std::ops::{Deref, DerefMut};

use tracing::{debug, error, warn};

use super::{ring::InsertPoint, Ring, Selector};

use crate::core::types::{Logical, Rectangle,};
use crate::platform::{Platform, PlatformWindowId};

/// A ring of Windows.
///
/// Contains additional methods more specific to window management.
///
/// It implements `Deref` and `DerefMut` to `Ring`, so you can
/// use all `Ring` methods on it.
///
/// The focused element of this ring is the window that currently
/// has the input focus.
///
/// A `WindowRing` also plays an important role in enforcing window
/// stacking, keeping all off-layout clients on top.
#[derive(Debug, Clone)]
pub struct WindowRing<P: Platform>(Ring<Window<P>>);
/* we still need to change focus on this everytime so we know
which window to cycle focus to */

impl<P: Platform> Deref for WindowRing<P> {
    type Target = Ring<Window<P>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P: Platform> DerefMut for WindowRing<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<P: Platform> Default for WindowRing<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Platform> WindowRing<P> {
    /// Creates a new ClientRing.
    pub fn new() -> Self {
        Self(Ring::new())
    }

    /// Adds the Client at a given index.
    pub fn add_at_index(&mut self, idx: usize, win: Window<P>) {
        self.insert(InsertPoint::Index(idx), win);
    }

    /// Wrapper around `Ring::remove` that takes a window ID instead of index.
    pub fn remove_by_id(&mut self, id: P::WindowId) -> Option<Window<P>> {
        let Some(i) = self.get_idx(id) else {
            return None
        };

        self.remove(i)
    }

    /// Wrapper around `Ring::index` that takes a window ID.
    pub fn get_idx(&self, id: P::WindowId) -> Option<usize> {
        self.index(Selector::Condition(&|win| win.id() == id))
    }

    /// Returns a reference to the client containing the given window ID.
    pub fn lookup(&self, id: P::WindowId) -> Option<&Window<P>> {
        if let Some(i) = self.get_idx(id) {
            self.get(i)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the client containing the given ID.
    pub fn lookup_mut(&mut self, id: P::WindowId) -> Option<&mut Window<P>> {
        self.get_idx(id).and_then(|i| self.get_mut(i))
    }

    /// Tests whether the Ring contains a client with the given ID.
    pub fn contains(&self, id: P::WindowId) -> bool {
        matches!(self.element_by(|win| win.id() == id), Some(_))
    }

    /// Sets the focused element to the given client.
    pub fn set_focused_by_winid(&mut self, id: P::WindowId) {
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
    pub fn is_focused(&self, id: P::WindowId) -> bool {
        if let Some(window) = self.focused() {
            window.id() == id
        } else {
            false
        }
    }
}

/// Represents a Window managed by a Toaru instance.
#[derive(Debug, Clone)]
pub struct Window<P: Platform> {
    pub(crate) id: P::WindowId,
    geom: Rectangle<i32, Logical>,

    initial_geom: Rectangle<i32, Logical>,
    urgent: bool,
    fullscreen: bool,

    // Indicates whether or not the Window is part of the current layout.
    inside_layout: bool,

    mapped: bool,
}

impl<P: Platform> PartialEq for Window<P> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<P: Platform> Window<P> {
    /// Creates a new Client from a given `id`.
    pub fn new(id: P::WindowId, geom: Option<Rectangle<i32, Logical>>) -> Self {
        let geom = geom.unwrap_or_else(|| Rectangle::zeroed());
        Self {
            id,
            geom,
            initial_geom: geom,
            urgent: false,
            fullscreen: false,
            inside_layout: false,
            mapped: false,
        }
    }

    /// Returns a Client that should float.
    pub fn outside_layout(from: P::WindowId, geom: Option<Rectangle<i32, Logical>>) -> Self {
        let mut new = Self::new(from, geom);
        new.inside_layout = false;

        new
    }

    /// Returns the X ID of the client.
    #[inline(always)]
    pub fn id(&self) -> P::WindowId {
        self.id
    }

    /// Returns the x coordinate of the window.
    #[inline(always)]
    pub fn x(&self) -> i32 {
        self.geom.point.x
    }

    /// Returns the y coordinate of the window.
    #[inline(always)]
    pub fn y(&self) -> i32 {
        self.geom.point.y
    }

    /// Returns the height of the window.
    #[inline(always)]
    pub fn height(&self) -> i32 {
        self.geom.size.height
    }

    /// Returns the width of the window.
    #[inline(always)]
    pub fn width(&self) -> i32 {
        self.geom.size.width
    }

    /// Returns the geometry of the window.
    #[inline(always)]
    pub fn geometry(&self) -> Rectangle<i32, Logical> {
        self.geom
    }

    /// Sets the geometry of the window.
    pub fn set_geometry(&mut self, geom: Rectangle<i32, Logical>) {
        self.geom = geom;
    }

    /// Returns the initial geometry of the window, as set by the
    /// program that created it.
    #[inline(always)]
    pub fn initial_geom(&self) -> Rectangle<i32, Logical> {
        self.initial_geom
    }

    /// Tests whether the Window's urgent flag is set.
    #[inline(always)]
    pub fn is_urgent(&self) -> bool {
        self.urgent
    }

    /// Returns whether the Window is fullscreen.
    #[inline(always)]
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Returns whether the Window is mapped.
    #[inline(always)]
    pub fn is_mapped(&self) -> bool {
        self.mapped
    }

    /// Returns whether the Window should be floated regardless
    /// of the current layout.
    #[inline(always)]
    pub fn is_off_layout(&self) -> bool {
        !self.inside_layout
    }

    /// Mark a Window as outside of the layout.
    pub fn set_off_layout(&mut self) {
        self.inside_layout = false;
    }

    /// Mark a Client as inside of the layout.
    pub fn set_on_layout(&mut self) {
        self.inside_layout = true;
    }
}

/// Maintains the focusing order of the windows of screen.
#[derive(Debug, Clone)]
pub(crate) struct FocusStack<C: PlatformWindowId>(Ring<C>);

impl<C: PlatformWindowId> Deref for FocusStack<C> {
    type Target = Ring<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: PlatformWindowId> DerefMut for FocusStack<C> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

#[allow(dead_code)]
impl<C: PlatformWindowId> FocusStack<C> {
    /// Creates a new FocusStack.
    pub fn new() -> Self {
        Self(Ring::new())
    }

    pub fn add_by_layout_status<P: Platform<WindowId = C>>(&mut self, id: C, clients: &WindowRing<P>) {
        let Some(cl) = clients.lookup(id) else {
            warn!("could not find client with id {:?} in clientring", id);
            return
        };

        if cl.is_off_layout() {
            self.push(id.clone());
        } else {
            let idx = self.partition_idx(clients);
            self.insert(InsertPoint::Index(idx), id.clone());
        }
    }

    pub fn set_focused_by_winid(&mut self, id: C) {
        if let Some(idx) = self.get_idx(id) {
            self.set_focused(idx);
        } else {
            warn!("No window with id {:?} found", id)
        }
    }

    pub fn remove_by_id(&mut self, id: C) -> Option<C> {
        self.get_idx(id).and_then(|idx| self.remove(idx))
    }

    pub fn on_layout<'ws, P: Platform<WindowId = C>>(&'ws self, cl: &'ws WindowRing<P>)
    -> impl Iterator<Item = &'ws C>
    {
        self.iter().filter(|id| {
            !(cl.lookup(**id)
                .expect("could not find client")
                .is_off_layout())
        })
    }

    pub fn off_layout<'ws, P: Platform<WindowId = C>>(&'ws self, cl: &'ws WindowRing<P>)
    -> impl Iterator<Item = &'ws C>
    {
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
    pub fn bubble_to_top<P: Platform<WindowId = C>>(&mut self, id: C, c: &WindowRing<P>) {
        if self.is_empty() {
            return;
        }
        let Some(idx) = c.get_idx(id) else {
            warn!("could not find window with ID {:?} in clientring", id);
            return
        };
        let Some(cl) = c.lookup(id) else {
            warn!("could not find window with ID {:?} in clientring", id);
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
    pub fn get_idx(&self, id: C) -> Option<usize> {
        self.0.index(Selector::Condition(&|win| *win == id))
    }

    /// Gets the index where the first window on layout resides.
    ///
    /// Assumes the `ClientRing` is indeed partitioned.
    //* precondition: the ring is already partitioned correctly */
    pub fn partition_idx<P: Platform<WindowId = C>>(&self, clients: &WindowRing<P>) -> usize {
        self.0
            .items
            .partition_point(|c| clients.lookup(*c).expect("no client found").is_off_layout())
    }
}
