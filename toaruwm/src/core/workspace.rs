//! Types used within workspaces.
//!
//! In ToaruWM, a Workspace represents a collection
//! of windows that can be displayed onscreen together, with a set
//! of layouts that can be swapped out or modified on the fly.
//!
//! The core type of this module is [`Workspace`].

use std::fmt;

use tracing::instrument;
use tracing::{debug, error, warn, trace};

use crate::core::{
    desktop::MonitorHandle, window::{Window, WindowRing, FocusStack},
};
use crate::layouts::{update::IntoUpdate, LayoutAction, LayoutType, Layouts};
use crate::config::MgrConfig;
use crate::types::Direction;
use crate::wayland::{WaylandWindow, WaylandWindowId};
use crate::Result;

/// A specification describing a workspace.
///
/// Each spec contains a name, a screen index, and the layouts
/// the workspace is to use. Each layout should correspond to
/// a Layout trait object within the overall configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceSpec {
    pub(crate) name: String,
    pub(crate) idx: usize,
    pub(crate) layouts: Vec<String>,
}

impl WorkspaceSpec {
    /// Creates a new WorkspaceSpec.
    pub fn new<S, L>(name: S, screen: usize, layouts: L) -> Self
    where
        S: Into<String>,
        L: IntoIterator<Item = String>,
    {
        Self {
            name: name.into(),
            idx: screen,
            layouts: layouts.into_iter().collect(),
        }
    }

    /// Getter method returning the specification's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Getter method returning the specification's screen.
    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Getter method returning the specification's layouts.
    pub fn layouts(&self) -> &[String] {
        &self.layouts
    }
}

/// A grouped collection of windows arranged according to a Layout.
///
/// # General Usage
///
/// Workspaces manage windows in two classes; within the layout
/// or outside the layout, marked by an attribute on the window.
/// windows within the layout are passed to the layout generator
/// to account for when it generates the layout, and windows outside
/// of the layout are always stacked on top and floated.
///
/// Most Workspace methods involve adding or removing windows, swapping
/// layouts, or modifying layouts.
///
/// ## Workspace Activation and Deactivation
/// 
/// Workspaces are displayed on monitors. When a workspace is displayed (i.e.
/// its windows are visible), it is considered **activated**. When it is not
/// being displayed on a monitor, it is considered **deactivated**.
/// 
/// When a workspace is activated, it holds a handle to a `Monitor` (i.e. a
/// [`MonitorHandle`]). When it is deactivated, it relinquishes this handle
/// to be passed to the next workspace to be displayed on the corresponding
/// `Monitor`.
/// 
/// `Workspace`s are activated and deactivated with the [`Workspace::activate`]
/// and [`Workspace::deactivate`] methods respectively.
///
/// # Layout
///
/// Workspaces have no notion of layout policy or layout-specific details,
/// such as the main and secondary windows on a dynamically tiled layout,
/// or for what reason certain windows are unmapped on a monocle-based
/// layout. They simply query the current focused layout and apply it,
/// or update the layouts as necessary.
///
/// See [`Layout`] for more information.
///
/// # Stacking Policy
///
/// While Workspaces have no notion of layout policy, they are aware
/// that there are layouts managing the placement of their windows.
/// Thus, they implement a stacking policy where any window off layout
/// is always stacked above. More precisely, Workspaces track two different
/// orders: The tiling order, which defines the arrangement of windows on
/// the screen, and the stacking order, which defines how windows are stacked
/// on top of each other.
///
/// # Panics
///
/// Any of `Workspace`'s layout-related methods may panic if any of
/// `Layouts`' invariants are not upheld.
///
/// See [`Layouts`] for more information.
/// 
/// [1]: crate::Toaru
pub struct Workspace {
    /// The workspace name.
    pub(crate) name: String,
    /// The windows currently in the workspace.
    pub(crate) windows: WindowRing,
    pub(crate) focuses: FocusStack,
    /// The layouts applied to this Workspace.
    pub(crate) layouts: Layouts,
    /// The current output this workspace is being displayed on, if any.
    pub(crate) output: Option<MonitorHandle>,
    pub(crate) config: MgrConfig,
}

impl fmt::Debug for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Workspace")
            .field("name", &self.name)
            .field("windows", &self.windows)
            .field("layouts", &Option::<u32>::None)
            .field("output", &self.output)
            .finish()
    }
}

impl Workspace {
    // * PUBLIC METHODS * //

    /// Creates a new workspace.
    pub fn new<S: Into<String>>(name: S, config: MgrConfig) -> Self {
        Self {
            name: name.into(),
            windows: WindowRing::new(),
            focuses: FocusStack::new(),
            layouts: Layouts::default(),
            output: None,
            config,
        }
    }

    /// Creates a new workspace with the provided `output`.
    pub fn new_with_output<S>(name: S, output: MonitorHandle, config: MgrConfig) -> Self
    where
        S: Into<String>
    {
        let mut wk = Self::new(name, config);
        wk.output = Some(output);
        wk
    }

    /// Creates a workspace from a given specification.
    pub fn from_spec(
        spec: WorkspaceSpec, 
        available_layouts: &Layouts, 
        output: Option<MonitorHandle>,
        config: MgrConfig,
    ) -> Result<Self> {
        let mut layouts = Vec::new();
        for name in spec.layouts {
            if let Some((_, l)) = available_layouts.element_by(|l| name == l.name()) {
                layouts.push(l.boxed());
            } else {
                error!("could not find layout with name {}", name);
            }
        }

        Ok(Self {
            name: spec.name,
            windows: WindowRing::new(),
            focuses: FocusStack::new(),
            layouts: Layouts::with_layouts_validated(layouts)?,
            output,
            config,
        })
    }

    /// If the Workspace is currently being shown on a monitor.
    pub fn is_active(&self) -> bool {
        self.output.is_some()
    }

    /// Set the active output on this workspace, returning the previous active output, if any.
    pub fn set_output(&mut self, output: MonitorHandle) -> Option<MonitorHandle> {
        self.output.replace(output)
    }

    /// Returns a reference to the current output the Workspace is shown on, if any.
    pub fn output(&self) -> Option<&MonitorHandle> {
        self.output.as_ref()
    }

    /// Sets the layout to use and applies it to all currently mapped windows.
    ///
    /// Is a no-op if no such layout exists.
    pub fn set_layout(&mut self, layout: &str) {
        let Some((idx, _)) = self.layouts.element_by(|ws| ws.name() == layout) else {
            warn!("No layout with name `{}`", layout);
            return
        };
        self.layouts.set_focused(idx);
        self.relayout();
    }

    /// Cycles in the given direction to the next layout, and applies it.
    pub fn cycle_layout(&mut self, dir: Direction) {
        self.layouts.cycle_focus(dir);
        self.relayout();
    }

    /// Switches to the given layout, and applies it.
    pub fn switch_layout<S>(&mut self, name: S)
    where
        S: AsRef<str>
    {
        if let Some((idx, _)) = self.layouts.element_by(|l| l.name() == name.as_ref()) {
            self.layouts.set_focused(idx);
            self.relayout();
        } else {
            error!("could not find layout {}", name.as_ref());
        }
    }

    /// Tests whether the workspace contains a specfic window.
    pub fn contains_window(&self, id: WaylandWindowId) -> bool {
        self.windows.contains(id)
    }

    /// Returns a reference to the currently focused client.
    pub fn focused(&self) -> Option<&Window> {
        self.windows.focused()
    }

    /// Returns a mutable reference to the currently focused client.
    pub fn focused_mut(&mut self) -> Option<&mut Window> {
        self.windows.focused_mut()
    }

    /// Returns the name of the workspace.
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns an iterator over all the windows in the workspace.
    #[inline]
    pub fn windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter()
    }

    /// Returns a mutable iterator over all the windows in the workspace.
    #[inline]
    pub fn windows_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        self.windows.iter_mut()
    }

    /// Returns an iterator over all the clients currently in the layout.
    #[inline]
    pub fn windows_in_layout(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter().filter(|w| !w.is_off_layout())
    }

    /// Returns a mutable iterator over all the clients currently in the layout.
    #[inline]
    pub fn windows_in_layout_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        self.windows.iter_mut().filter(|w| !w.is_off_layout())
    }

    /// Returns an iterator over all the clients currently off the layout.
    #[inline]
    pub fn windows_off_layout(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter().filter(|w| w.is_off_layout())
    }

    /// Returns a mutable iterator over all the clients currently off the layout.
    #[inline]
    pub fn windows_off_layout_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        self.windows.iter_mut().filter(|w| w.is_off_layout())
    }

    /// Tests whether the workspace is empty.`
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Tests whether the workspace is tiled.
    #[inline(always)]
    pub fn is_tiling(&self) -> bool {
        !self.is_floating()
    }

    /// Tests whether the workspace is floating.
    #[inline]
    pub fn is_floating(&self) -> bool {
        // just assume the invariants hold
        matches!(self.layout_style(), LayoutType::Floating)
    }

    /// Returns the name of the workspace's current layout.
    #[inline]
    pub fn layout(&self) -> &str {
        let layout = self
            .layouts
            .focused()
            .expect("layout focus should not be None");
        layout.name()
    }

    /// Returns the style of the workspace's current layout.
    #[inline]
    pub fn layout_style(&self) -> LayoutType {
        let layout = self
            .layouts
            .focused()
            .expect("layout focus should not be None");
        layout.style()
    }

    /// Returns the `Some(idx)` where `idx` is the index of the
    /// Window in its underlying ring, or `None` if the Window
    /// does not exist.
    #[inline]
    pub fn contains(&self, window: WaylandWindowId) -> Option<usize> {
        self.windows.get_idx(window)
    }

    /// Activates the workspace, mapping all windows in it, and generating the layout
    /// for windows to follow.
    ///
    /// The window that gets the focus in the one that is currently
    /// focused in the internal Ring.
    pub fn activate(&mut self, mon: MonitorHandle) -> Option<MonitorHandle> {
        if self.windows.is_empty() {
            return self.output.replace(mon);
        }

        self.windows.iter_mut().for_each(|w| w.map());

        let ret = self.output.replace(mon);
        self.relayout();

        if let Some(win) = self.focused() {
            self.focus_window(win.id());
        } else {
            debug!("no focused window, focusing by ptr");
            self.focus_window_by_ptr();
        }

        ret
    }

    /// Unmaps all the windows in the workspace and sets the workspace to inactive,
    /// relinquishing the MonitorHandle.
    pub fn deactivate(&mut self) -> MonitorHandle {
        self.windows.iter_mut().for_each(|w| w.unmap());

        self.output.take().expect("Cannot deactivate an output with no active output")
    }

    /// Take the current output of the workspace without deactivating it.
    pub fn take_output(&mut self) -> Option<MonitorHandle> {
        self.output.take()
    }

    /// Calls the layout function and applies it to the workspace.
    pub fn relayout(&mut self) {
        if !self.output.is_none() {
            return;
        }
        let layouts = self.layouts.gen_layout(self, &self.config);
        self.apply_layout(&layouts);
    }

    /// Adds a window to the workspace in the layout.
    pub fn add_window_on_layout(&mut self, window: WaylandWindow) {
        self._add_window(Window::new(window, None))
    }

    /// Adds a window to the workspace off the layout.
    pub fn add_window_off_layout(&mut self, window: WaylandWindow) {
        self._add_window(Window::outside_layout(window, None))
    }

    /// Deletes the window from the workspaces and returns it.
    pub fn del_window(&mut self,id: WaylandWindowId) -> Option<Window> {
        let Some(win) = self.windows.lookup(id) else {
            warn!("No window with id {id:?} found");
            return None
        };

        if win.is_off_layout() {
            Some(self._del_window(id, false))
        } else {
            Some(self._del_window(id, true))
        }
    }

    /// Deletes the focused window in the workspace and returns it.
    pub fn del_focused_window(&mut self) -> Option<Window> {
        self.windows.focused()
            .map(|win| win.id())
            .and_then(|id| self.del_window(id))
    }

    /// Takes a window directly without calling the layout.
    pub fn take_window(&mut self, window: WaylandWindowId) -> Option<Window> {
        self.windows.remove_by_id(window)
    }

    /// Takes the focused window directly without calling the layout.
    pub fn take_focused_window(&mut self) -> Option<Window> {
        self.windows.focused()
            .map(|win| win.id())
            .and_then(|id| self.take_window(id))
    }

    /// Sets the focused window to the given ID.
    ///
    /// Also calls `Self::unfocus_window` internally.
    pub fn focus_window(&mut self, window: WaylandWindowId) {
        let Some(_) = self.windows.get_idx(window) else {
            warn!("focus_window: no window {:?} found in workspace", window);
            return
        };

        self.windows.set_focused_by_winid(window);
    }

    /// Cycles the focus to the next window in the workspace.
    pub fn cycle_focus(&mut self, dir: Direction) {
        // get the currently focused window's ID
        if self.windows.focused_mut().is_none() {
            error!("cycle_focus for ws {}: nothing focused", self.name);
            return;
        };

        self.windows.cycle_focus(dir);
    }

    /// Toggles fullscreen on the currently focused window.
    pub fn toggle_focused_fullscreen(&mut self) {
        self.windows.focused_mut().map(|w| w.toggle_fullscreen());
    }

    /// Toggles the state of the currently focused window between off or in layout.
    pub fn toggle_focused_state<C>(&mut self) {
        // If we have a focused window
        if let Some(win) = self.windows.focused() {
            debug!("toggling state of focused window {:?}", win.id());
            if win.is_off_layout() {
                self.add_to_layout(win.id())
            } else {
                self.remove_from_layout(win.id())
            }
        }
    }

    /// Sets the focused window to be managed by the layout.
    ///
    /// Is effectively a no-op if the workspace is in a floating-style layout.
    pub fn add_to_layout(&mut self, id: WaylandWindowId) {
        debug!("Setting focused to tiled");

        if let Some(win) = self.windows.lookup_mut(id) {
            win.set_on_layout();
            self.focuses.bubble_to_top(id, &self.windows);
            self.relayout();
        }
    }

    /// Removes the focused window from being managed by the layout, effectively
    /// turning it into a floating window regardless of the current layout style.
    ///
    /// This will also stack the window above any other windows.
    pub fn remove_from_layout(&mut self, id: WaylandWindowId) {
        debug!("removing {:?} from layout", id);
        if let Some(win) = self.windows.lookup_mut(id) {
            win.set_off_layout();
            self.focuses.bubble_to_top(id, &self.windows);
            self.relayout();
        }
    }

    /// Sends an update to the currently focused layout, and applies
    /// and changes that may have taken place.
    pub fn update_focused_layout<U: IntoUpdate>(&mut self, msg: U) {
        self.layouts.send_update(msg.into_update());
        self.relayout();
    }

    /// Checks if the Window with given `id` is managed under layout.
    pub fn has_window_in_layout(&self, id: WaylandWindowId) -> bool {
        self.windows_in_layout().any(|c| c.id() == id)
    }

    /// Returns the number of windows managed by the layout.
    ///
    /// Since a workspace can contain both floating and tiled windows,
    /// this returns the number of tiled windows only.
    pub fn managed_count(&self) -> usize {
        self.windows_in_layout().count()
    }

    /// Returns the number of floating windows in the workspace.
    ///
    /// Since a workspace can contain both floating and tiled windows,
    /// this returns the number of floating windows only.
    pub fn floating_count(&self) -> usize {
        self.windows_off_layout().count()
    }

    //* ========================================= *//
    // *             PRIVATE METHODS             * //
    //* ========================================= *//

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip_all))]
    fn _add_window(&mut self, window: Window) {
        trace!("adding window {:#?}", window);
        self.windows.append(window);
        self.relayout();
    }

    /// Deletes a window
    fn _del_window(&mut self, id: WaylandWindowId, on_layout: bool,) -> Window {
        let Some(window) = self.windows.remove_by_id(id) else {
            error!("Tried to remove window with {:?} but it does not exist", id);
            panic!("AAAAAA"); //fixme
        };
        self.focuses.remove_by_id(id);

        // the WindowRing should cycle to a new focused when remove our window
        if let Some(win) = self.windows.focused() {
            // self.stack_and_focus_window(pf, cfg, win.id());
        }

        // if empty, no need to unset focused, the WindowRing will do that for us

        if on_layout {
            self.relayout();
        }

        window
    }

    /// Pushes a window directly without calling the layout.
    pub(crate) fn put_window(&mut self, window: Window) {
        let id = window.id();
        self.windows.push(window);
        self.focuses.add_by_layout_status(id, &self.windows);
    }

    /// Updates the focus to the window under the pointer.
    pub(crate) fn focus_window_by_ptr(&mut self) {
        //todo: make all methods return Result
        // let Ok(reply) = pf.query_pointer(scr.root_id) else {
        //     warn!("could not query pointer");
        //     return
        // };
        // self.focus_window(reply.child, pf, cfg);
    }

    fn apply_layout(&mut self, layouts: &[LayoutAction]) {
        for rsaction in layouts {
            match rsaction {
                LayoutAction::Resize { id, geom } => {
                    let window = self.windows.lookup_mut(*id).unwrap();
                    window.set_geometry(*geom);
                }
                LayoutAction::StackOnTop(id) => {
                    self.focuses.bubble_to_top(*id, &self.windows);
                }
                LayoutAction::Remove(id) => {
                    let window = self.windows.lookup_mut(*id).unwrap();
                    window.set_off_layout();
                }
                _ => {}
            }
        }
    }
}
