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
    desktop::Monitor, window::{Window, WindowRing, FocusStack},
};
use crate::layouts::{update::IntoUpdate, LayoutAction, LayoutType, Layouts};
use crate::config::RuntimeConfig;
use crate::types::Direction;
use crate::platform::{Platform};

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
/// # Common Method Parameters
///
/// You may notice that a lot of the methods on `Workspace` have many
/// scary-looking trait bounds; this is because they are also
/// generic over the two types [`Toaru`][1] itself is generic over:
/// an [`Platform`], and a [`RuntimeConfig`]. These two provide an interface
/// that the `Workspace` needs to perform a lot of its functionality.
///
/// Generally speaking, if you're using `WindowManager` itself, chances
/// are you won't have to call many of these methods directly.
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
pub struct Workspace<P>
where
    P: Platform,
{
    /// The workspace name.
    pub(crate) name: String,
    /// The windows currently in the workspace.
    pub(crate) windows: WindowRing<P>,
    pub(crate) focuses: FocusStack<P::WindowId>,
    /// The layouts applied to this Workspace.
    pub(crate) layouts: Layouts<P>,
    /// The current output this workspace is being displayed on, if any.
    pub(crate) output: Option<P::Output>,
}

impl<P: Platform> fmt::Debug for Workspace<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Workspace")
            .field("name", &self.name)
            .field("windows", &self.windows)
            .field("layouts", &Option::<u32>::None)
            .finish()
    }
}

impl<P: Platform> Workspace<P> {
    // * PUBLIC METHODS * //

    /// Creates a new workspace.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            windows: WindowRing::new(),
            focuses: FocusStack::new(),
            layouts: Layouts::default(),
            output: None,
        }
    }

    /// Creates a new workspace with the provided `output`.
    pub fn new_with_output<S: Into<String>>(name: S, output: P::Output) -> Self {
        let mut wk = Self::new(name);
        wk.output = Some(output);
        wk
    }

    /// Creates a workspace from a given specification.
    pub fn from_spec(spec: WorkspaceSpec, available_layouts: &Layouts<P>, output: Option<P::Output>) -> Result<Self> {
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
        })
    }

    /// If the Workspace is currently being shown on a monitor.
    pub fn is_active(&self) -> bool {
        self.output.is_some()
    }

    /// Set the active output on this workspace, returning the previous active output, if any.
    pub fn set_output(&mut self, output: P::Output) -> Option<P::Output> {
        self.output.replace(output)
    }

    /// Sets the layout to use and applies it to all currently mapped windows.
    ///
    /// Is a no-op if no such layout exists.
    pub fn set_layout<C>(&mut self, layout: &str, scr: &Monitor<P>, cfg: &C)
    where
        C: RuntimeConfig,
    {
        let Some((idx, _)) = self.layouts.element_by(|ws| ws.name() == layout) else {
            warn!("No layout with name `{}`", layout);
            return
        };
        self.layouts.set_focused(idx);
        self.relayout(scr, cfg);
    }

    /// Cycles in the given direction to the next layout, and applies it.
    pub fn cycle_layout<C>(&mut self, dir: Direction, scr: &Monitor<P>, cfg: &C)
    where
        C: RuntimeConfig,
    {
        self.layouts.cycle_focus(dir);
        self.relayout(scr, cfg);
    }

    /// Switches to the given layout, and applies it.
    pub fn switch_layout<S, C>(&mut self, name: S, scr: &Monitor<P>, cfg: &C)
    where
        S: AsRef<str>,
        C: RuntimeConfig,
    {
        if let Some((idx, _)) = self.layouts.element_by(|l| l.name() == name.as_ref()) {
            self.layouts.set_focused(idx);
            self.relayout(scr, cfg);
        } else {
            error!("could not find layout {}", name.as_ref());
        }
    }

    /// Tests whether the workspace contains a specfic window.
    pub fn contains_window(&self, id: P::WindowId) -> bool {
        self.windows.contains(id)
    }

    /// Returns a reference to the currently focused client.
    pub fn focused(&self) -> Option<&Window<P>> {
        self.windows.focused()
    }

    /// Returns a mutable reference to the currently focused client.
    pub fn focused_mut(&mut self) -> Option<&mut Window<P>> {
        self.windows.focused_mut()
    }

    /// Returns the name of the workspace.
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns an iterator over all the windows in the workspace.
    #[inline]
    pub fn windows(&self) -> impl Iterator<Item = &Window<P>> {
        self.windows.iter()
    }

    /// Returns a mutable iterator over all the windows in the workspace.
    #[inline]
    pub fn windows_mut(&mut self) -> impl Iterator<Item = &mut Window<P>> {
        self.windows.iter_mut()
    }

    /// Returns an iterator over all the clients currently in the layout.
    #[inline]
    pub fn windows_in_layout(&self) -> impl Iterator<Item = &Window<P>> {
        self.windows.iter().filter(|w| !w.is_off_layout())
    }

    /// Returns a mutable iterator over all the clients currently in the layout.
    #[inline]
    pub fn windows_in_layout_mut(&mut self) -> impl Iterator<Item = &mut Window<P>> {
        self.windows.iter_mut().filter(|w| !w.is_off_layout())
    }

    /// Returns an iterator over all the clients currently off the layout.
    #[inline]
    pub fn windows_off_layout(&self) -> impl Iterator<Item = &Window<P>> {
        self.windows.iter().filter(|w| w.is_off_layout())
    }

    /// Returns a mutable iterator over all the clients currently off the layout.
    #[inline]
    pub fn windows_off_layout_mut(&mut self) -> impl Iterator<Item = &mut Window<P>> {
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
    pub fn contains(&self, window: P::WindowId) -> Option<usize> {
        self.windows.get_idx(window)
    }

    /// Maps all the windows in the workspace.
    ///
    /// The window that gets the focus in the one that is currently
    /// focused in the internal Ring.
    #[cfg_attr(debug_assertions, instrument(level = "debug", skip_all))]
    pub fn activate<C>(&mut self, scr: &Monitor<P>, cfg: &C) -> Option<P::Output>
    where
        C: RuntimeConfig,
    {
        if self.windows.is_empty() {
            return self.output.replace(scr.output.clone());
        }

        self.windows.iter_mut().for_each(|w| w.map());

        self.relayout(scr, cfg);

        if let Some(win) = self.focused() {
            self.focus_window(win.id(), cfg);
        } else {
            debug!("no focused window, focusing by ptr");
            self.focus_window_by_ptr(scr, cfg);
        }

        self.output.replace(scr.output.clone())
    }

    /// Unmaps all the windows in the workspace and sets the workspace to inactive.
    pub fn deactivate(&mut self) -> P::Output {
        self.windows.iter_mut().for_each(|w| w.unmap());

        self.output.take().expect("Cannot deactivate an output with no active output")
    }

    /// Calls the layout function and applies it to the workspace.
    pub fn relayout<C>(&mut self, scr: &Monitor<P>, cfg: &C)
    where
        C: RuntimeConfig,
    {
        if !self.output.is_none() {
            return;
        }
        let layouts = self.layouts.gen_layout(self, scr, cfg);
        self.apply_layout(&layouts);
    }

    /// Adds a window to the workspace in the layout.
    pub fn add_window_on_layout<C>(&mut self, window: P::Window, scr: &Monitor<P>, cfg: &C)
    where
        C: RuntimeConfig
    {
        self._add_window(scr, cfg, Window::new(window, None))
    }

    /// Adds a window to the workspace off the layout.
    pub fn add_window_off_layout<C: RuntimeConfig>(
        &mut self,
        window: P::Window,
        scr: &Monitor<P>,
        cfg: &C,
    )
    {
        self._add_window(scr, cfg, Window::outside_layout(window, None))
    }

    /// Deletes the window from the workspaces and returns it.
    pub fn del_window<C: RuntimeConfig>(
        &mut self,
        id: P::WindowId,
        scr: &Monitor<P>,
        cfg: &C,
    ) -> Option<Window<P>> {
        if let Some(win) = self.windows.lookup(id) {
            if win.is_off_layout() {
                Some(self._del_window(scr, cfg, id, false))
            } else {
                Some(self._del_window(scr, cfg, id, true))
            }
        } else {
            // fail silently (this accounts for spurious unmap events)
            debug!("could not find window to delete, failing silently");
            None
        }
    }

    /// Sets the focused window to the given ID.
    ///
    /// Also calls `Self::unfocus_window` internally.
    pub fn focus_window<C>(&mut self, window: P::WindowId, cfg: &C)
    where
        C: RuntimeConfig
    {
        let Some(_) = self.windows.get_idx(window) else {
            warn!("focus_window: no window {:?} found in workspace", window);
            return
        };

        debug!("found window {:?}", window);
        if let Some(focused) = self.windows.focused_mut() {
            let id = focused.id();
            //self.unfocus_window(id, pf, cfg);
        }
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

    /// Deletes the focused window in the workspace and returns it.
    pub fn take_focused_window<C>(
        &mut self,
        screen: &Monitor<P>,
        cfg: &C,
    ) -> Option<Window<P>>
    where
        C: RuntimeConfig,
    {
        if let Some(window) = self.windows.focused() {
            let id = window.id();
            self.del_window(id, screen, cfg)
        } else {
            None
        }
    }

    /// Toggles fullscreen on the currently focused window.
    pub fn toggle_focused_fullscreen(&mut self) {
        self.windows.focused_mut().map(|w| w.toggle_fullscreen());
    }

    /// Toggles the state of the currently focused window between off or in layout.
    pub fn toggle_focused_state<C>(&mut self, scr: &Monitor<P>, cfg: &C)
    where
        P: Platform,
        C: RuntimeConfig,
    {
        // If we have a focused window
        if let Some(win) = self.windows.focused() {
            debug!("toggling state of focused window {:?}", win.id());
            if win.is_off_layout() {
                self.add_to_layout(win.id(), scr, cfg)
            } else {
                self.remove_from_layout(win.id(), scr, cfg)
            }
        }
    }

    /// Sets the focused window to be managed by the layout.
    ///
    /// Is effectively a no-op if the workspace is in a floating-style layout.
    pub fn add_to_layout<C>(&mut self, id: P::WindowId, scr: &Monitor<P>, cfg: &C)
    where
        P: Platform,
        C: RuntimeConfig,
    {
        debug!("Setting focused to tiled");

        if let Some(win) = self.windows.lookup_mut(id) {
            win.set_on_layout();
            self.focuses.bubble_to_top(id, &self.windows);
            self.relayout(scr, cfg);
        }
    }

    /// Removes the focused window from being managed by the layout, effectively
    /// turning it into a floating window regardless of the current layout style.
    ///
    /// This will also stack the window above any other windows.
    pub fn remove_from_layout<C>(&mut self, id: P::WindowId, scr: &Monitor<P>, cfg: &C)
    where
        P: Platform,
        C: RuntimeConfig,
    {
        debug!("removing {:?} from layout", id);
        if let Some(win) = self.windows.lookup_mut(id) {
            win.set_off_layout();
            self.focuses.bubble_to_top(id, &self.windows);
            self.relayout(scr, cfg);
        }
    }

    /// Sends an update to the currently focused layout, and applies
    /// and changes that may have taken place.
    pub fn update_focused_layout<U: IntoUpdate, C>(
        &mut self,
        msg: U,
        scr: &Monitor<P>,
        cfg: &C,
    ) where
        P: Platform,
        C: RuntimeConfig,
    {
        self.layouts.send_update(msg.into_update());
        self.relayout(scr, cfg);
    }

    /// Checks if the Window with given `id` is managed under layout.
    pub fn has_window_in_layout(&self, id: P::WindowId) -> bool {
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
    fn _add_window<C>(&mut self, scr: &Monitor<P>, cfg: &C, window: Window<P>)
    where
        C: RuntimeConfig,
    {
        trace!("adding window {:#?}", window);
        self.windows.append(window);
        self.relayout(scr, cfg);
    }

    /// Deletes a window
    fn _del_window<C>(
        &mut self,
        scr: &Monitor<P>,
        cfg: &C,
        id: P::WindowId,
        on_layout: bool,
    ) -> Window<P>
    where
        P: Platform,
        C: RuntimeConfig,
    {
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
            self.relayout(scr, cfg);
        }

        window
    }

    /// Pushes a window directly without calling the layout.
    pub(crate) fn put_window(&mut self, window: Window<P>) {
        let id = window.id();
        self.windows.push(window);
        self.focuses.add_by_layout_status(id, &self.windows);
    }

    /// Takes a window directly without calling the layout.
    pub(crate) fn take_window(&mut self, window: P::WindowId) -> Option<Window<P>> {
        let window = self.windows.remove_by_id(window)?;
        Some(window)
    }

    /// Updates the focus to the window under the pointer.
    pub(crate) fn focus_window_by_ptr<C>(&mut self, scr: &Monitor<P>, cfg: &C)
    where
        C: RuntimeConfig,
    {
        //todo: make all methods return Result
        // let Ok(reply) = pf.query_pointer(scr.root_id) else {
        //     warn!("could not query pointer");
        //     return
        // };
        // self.focus_window(reply.child, pf, cfg);
    }

    fn apply_layout(&mut self, layouts: &[LayoutAction<P>]) {
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
