use std::fmt;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::core::{
    Monitor, WorkspaceSpec, Window, Workspace, WorkspaceMux
};
use crate::layouts::{update::IntoUpdate, Layout, Layouts};
use crate::types::{Cardinal, Direction, Rectangle, Point, Logical};
use crate::platform::{Platform};
use crate::config::{ManagerConfig, RuntimeConfig, MgrConfig};

use crate::{Result, ToaruError};
use super::ToaruState;


/// The main object that defines window management functionality.
/// 
/// `Toaru` abstracts over shared commonality between the Wayland and X11 
/// protocols, presenting a unified interface that you can use to manage windows 
/// in a platform-agnostic manner.
///
/// `Toaru` is generic over two types:
///
/// - `P`, that is its backing platform and so must implement the [`Platform`]
/// trait.
/// 
/// - `C`, that is its runtime configuration and must implement the [`RuntimeConfig`]
/// trait. This stores all configuration during the window manager's lifetime, and 
/// holds both information defined by this crate, as well as user-defined data.
///
/// These two traits are _central_ to the operation of a window manager, and as such 
/// you will see them pop up in a lot of places, mostly `Workspace` or `Desktop` 
/// methods, but also the occasional `Client` method.
///
/// # Structure and Management Model
///
/// Toaru's window management model involves a set of uniquely named workspaces, multiplexed
/// between a set of monitors. At any given time, if a monitor is active, it has a workspace
/// active on it, which manages windows, both mapped and unmapped.
///
/// # Relationship between `Toaru` and its `Platform`
/// 
/// `Toaru` serves as your interface to the platform. You manipulate its state 
/// through your keybind callbacks or in your code, and the `Platform` implements it
/// for you. Any eye-candy that the `Platform` might implement is transparent to `Toaru`.
/// That is, assuming `Toaru` implements its logic correctly, all window operations
/// on it should be atomic, and there is no such thing as inconsistent state between any
/// two window operations, as presented to the `Platform`.
/// 
/// Take for example, the Platform is in the middle of running an animation, transitioning
/// from one workspace to another, when a new window opens in the destination workspace.
/// By the time the new window opens, the internal `Toaru` state is already at the destination,
/// and the new window event is transmitted to `Toaru`, which seamlessly accounts for it.
/// Between these two events, there is no inconsistent state such as `Toaru` being halfway between
/// workspaces.
/// 
/// However, it is a different case in the backing `Platform`, where there is such a thing.
/// When a new window opens in a platform, it might play its own animation, and such there
/// is a conflict between the new animation that must play and the currently playing animation.
/// Resolving this conflict is transparent to `Toaru`, and is a `Platform`-level policy.
///
/// [1]: crate::core::Workspace
pub struct Toaru<P, C>
where
    P: Platform,
    C: RuntimeConfig,
{
    /// The internal config of the WindowManager.
    config: MgrConfig,
    /// The workspaces.
    workspaces: WorkspaceMux<P>,
    /// All screens connected to the computer.
    monitors: HashMap<P::Output, Monitor<P>>,
    /// The window currently being manipulated
    /// if `self.mousemode` is not None.
    selected: Option<P::WindowId>,

    _cfg_phantom: PhantomData<C>,
}

/// General `WindowManager`-level commands.
impl<P, C> Toaru<P, C>
where
    P: Platform,
    C: RuntimeConfig
{
    /// Constructs a new Toaru object.
    ///
    /// This method is simply generic over any type that implements
    /// [`Config`], since the trait bounds on `W` and `L` are already
    /// enforced by this trait. As long as `config` implements
    /// `Config`, it will work.
    ///
    /// # Assumptions
    ///
    /// This method assumes `config` has already been validated.
    /// It is on you to prevalidate your configuration and ensure
    /// all your invariants are upheld.
    ///
    /// See [`Config`] for more details.
    pub fn new<E, W, L>(mut config: E) -> Result<Toaru<P, C>>
    where
        E: ManagerConfig<P, Runtime = C, Workspaces = W, Layouts = L>,
        W: IntoIterator<Item = WorkspaceSpec>,
        L: IntoIterator<Item = Box<dyn Layout<P>>>,
    {
        let specs: Vec<WorkspaceSpec> = config.take_workspaces().into_iter().collect();
        let layouts = Layouts::with_layouts_validated(
            config.take_layouts()
                .into_iter()
                .collect::<Vec<Box< dyn Layout<P>>>>()   
        )?;

        let mut wksps = Vec::new();

        let config = config.into_runtime_config().into_managerconfig();

        for spec in specs.into_iter().rev() {
            wksps.push(Workspace::from_spec(spec, &layouts, None, config.clone())?);
        }

        let workspaces = WorkspaceMux::new(wksps, config.clone())?;

        Ok(Self {
            config,
            workspaces,
            monitors: HashMap::new(),
            selected: None,
            _cfg_phantom: PhantomData,
        })
    }

    /// Returns a reference to the internal runtime configuration of Toaru.
    pub fn config(&self) -> &C {
        &self.config.downcast()
    }

    /// Returns a new `ManagerConfig` that points to its internal runtime configuration.
    pub fn get_managerconfig(&self) -> MgrConfig {
        self.config.clone()
    }

    /// Provides a ToaruState for introspection.
    pub fn state(&self, monitor: &P::Output) -> ToaruState<'_, P, C> {
        let mon = self.monitors.get(monitor).expect("output should already be present");
        ToaruState {
            config: &self.config.downcast(),
            monitor: mon,
            selected: self.selected.as_ref(),
        }
    }

    /// Creates a new window and inserts it into the currently focused workspace.
    pub fn insert_window(&mut self, id: P::WindowId, output: Option<P::Output>) {
        todo!()
    }

    /// Removes the window identified by `id`.
    pub fn delete_window(&mut self, id: P::WindowId) -> Window<P> {
        todo!()
    }

    /// Configures a window with a given `id`.
    pub fn configure_window(&mut self, id: P::WindowId, geom: Rectangle<i32, Logical>) {
        todo!()
    }

    /// Maps the window, configuring it within its workspace.
    pub fn map_window(&mut self, id: P::WindowId) {
        todo!()
    }

    /// Unmaps the window.
    pub fn unmap_window(&mut self, id: P::WindowId) {
        todo!()
    }

    /// Add a new output to Toaru.
    pub fn add_output(&mut self, output: P::Output) {
        let idx = self.monitors.len();

        let monitor = Monitor::new(output.clone(), &self.workspaces, idx as i32);

        self.monitors.insert(output, monitor);
    }

    /// Gets the monitor with the provided `PlatformOutput`.
    pub fn get_output(&mut self, output: &P::Output) -> Option<&mut Monitor<P>> {
        self.monitors.get_mut(output)
    }

    /// Remove an output from Toaru.
    pub fn remove_output(&mut self, output: &P::Output) -> Option<Monitor<P>> {
        todo!()
    }

    /// Runs a closure on all workspaces managed within Toaru.
    pub fn with_workspaces<F, T>(&mut self, f: F) -> T 
    where
        F: FnOnce(&mut [Workspace<P>]) -> T
    {
        self.workspaces.with_workspaces(f)
    }

    /// Run a closure on for each workspace managed within Toaru.
    /// 
    /// If `active_only` is true, the closure is run only for active workspaces.
    pub fn foreach_workspace<F>(&mut self, active_only: bool, f: F)
    where
        F: FnMut(&mut Workspace<P>)
    {
        self.workspaces.foreach_wksp(active_only, f);
    }
}

/// Desktop-level commands.
impl<P, C> Toaru<P, C>
where
    P: Platform<Error = ToaruError>,
    C: RuntimeConfig,
{   
    /// Goes to the specified workspace on the currently active monitor.
    pub fn goto_workspace(&mut self, name: &str) {
        todo!()
    }

    /// Cycles the focused workspace on the currently active monitor.
    pub fn cycle_workspace(&mut self, direction: Direction) {
        todo!()
    }

    /// Sends the focused window to the specified workspace.
    pub fn send_focused_to(&mut self, name: &str, switch: bool) {
        todo!()
    }
}

/// Workspace-level commands.
impl<P, C> Toaru<P, C>
where
    P: Platform,
    C: RuntimeConfig,
{
    /// Cycles the focused window.
    pub fn cycle_focus(&mut self, direction: Direction) {
        todo!()
    }

    /// Cycles in the given direction to the layout applied to the current workspace.
    pub fn cycle_layout(&mut self, direction: Direction) {
        todo!()
    }

    /// Toggles the state of the focused window to floating or vice versa.
    pub fn toggle_focused_floating(&mut self) {
        todo!()
    }

    /// Sends an [`Update`](crate::layouts::update::Update)
    /// to the current layout.
    pub fn update_current_layout<U: IntoUpdate>(&mut self, update: U) {
        todo!()
    }

    /// Switches to the given layout on the current workspace.
    pub fn switch_layout<S: AsRef<str>>(&mut self, name: S) {
        todo!()
    }

    /// Toggles the focused window to fullscreen.
    pub fn toggle_focused_fullscreen(&mut self) {
        todo!()
    }

    /// Grabs the pointer and moves the window the pointer is on.
    ///
    /// If the selected window is under layout, it is removed from
    /// layout and the entire workspace is then re-laid out.
    //#[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    pub fn move_window_ptr(&mut self, pt: Point<i32, Logical>) {

    }

    /// Grabs the pointer and resizes the window the pointer is on.
    ///
    /// If the selected window is under layout, it is removed from
    /// layout and the entire workspace is then re-laid out.
    pub fn resize_window_ptr(&mut self, pt: Point<i32, Logical>) {

    }

    /// Moves the window `delta` pixels in direction `dir`.
    pub fn move_window(&mut self, delta: i32, dir: Cardinal) {
        
    }

    /// Resizes the window `delta` pixels in direction `dir`.
    pub fn resize_window(&mut self, delta: i32, dir: Cardinal) {
        
    }

    /// Closes the focused window.
    pub fn close_focused_window(&mut self) {
        
    }
}

#[doc(hidden)]
impl<P: Platform, C: RuntimeConfig> Toaru<P, C> {

}

impl<P, C> fmt::Debug for Toaru<P, C>
where
    P: Platform,
    C: RuntimeConfig + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WindowManager")
            .field("config", &self.config)
            .field("workspaces", &self.workspaces)
            .field("screens", &self.monitors)
            .field("selected", &self.selected)
            .finish()
    }
}