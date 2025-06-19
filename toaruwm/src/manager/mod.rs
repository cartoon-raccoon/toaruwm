//! The window manager itself, and associated modules.

#![allow(unused_variables, unused_imports)] //fixme

use std::ffi::OsStr;
use std::fmt;
use std::iter::FromIterator;
use std::process::{Command, Stdio};
use std::marker::PhantomData;

//use std::marker::PhantomData;

//use std::sync::OnceLock;

use tracing::instrument;
use tracing::{debug, error, info, span, warn, Level};

use crate::bindings::{Keybind, Keybinds, Mousebind, Mousebinds};
use crate::core::{Desktop, Screen, WorkspaceSpec};
use crate::layouts::{update::IntoUpdate, Layout, Layouts};
use crate::log::DefaultErrorHandler;
use crate::types::{Cardinal, Direction, Point, Ring, Selector, ClientId};
use crate::platform::{Platform};

use crate::{ErrorHandler, Result, ToaruError};

pub mod config;
/// A translation layer for converting X events into `WindowManager` actions.
pub mod event;
/// Macros and storage types for window manager hooks.
pub mod hooks;
pub mod state;

#[doc(inline)]
pub use config::{Config, ToaruConfig};
#[doc(inline)]
pub use event::EventAction;
#[doc(inline)]
pub use hooks::{Hook, Hooks};
#[doc(inline)]
pub use state::{RuntimeConfig, ToaruState};

//static ERR_HANDLER: OnceLock<&dyn FnMut(ToaruError)> = OnceLock::new();

macro_rules! handle_err {
    ($call:expr, $_self:expr) => {
        if let Err(e) = $call {
            $_self.ehandler.call($_self.state(), e.into());
        }
    };
}

/// Removes the focused window if under layout.
macro_rules! _rm_if_under_layout {
    ($_self:expr, $id:expr) => {
        let is_under_layout = $_self.desktop.current().has_window_in_layout($id);

        if is_under_layout {
            $_self.desktop.current_mut().remove_from_layout(
                &$_self.platform.handle(),
                $id,
                $_self.screens.focused().unwrap(),
                &$_self.config,
            );
        }
    };
}

/// The main object that defines client management functionality.
///
/// `Toaru` is generic over two types:
///
/// - P, that is its backing platform and so must implement
/// the [`Platform`] trait. This is the type by which `Toaru`
/// connects to the X server, and receives events and issues requests.
///
/// - C, that is its runtime configuration and must implement the
/// [`RuntimeConfig`] trait. This stores all configuration during
/// the window manager's lifetime, and holds both information
/// defined by this crate, as well as user-defined data.
///
/// These two traits are _central_ to the operation of a window manager,
/// and as such you will see them pop up in a lot of places, mostly
/// `Workspace` or `Desktop` methods, but also the occasional
/// `Client` method.
///
/// # Structure
///
/// This type combines a [`Platform`] and a [`Desktop`] which
/// combines [`Workspace`][1]s. As the top-level struct, `Toaru`
/// has methods that apply to its sub-structures, and thus are
/// organized accordingly: top-level, `Desktop`-level,
/// and `Workspace`-level.
///
/// # Usage
///
pub struct Toaru<P, C>
where
    P: Platform,
    C: RuntimeConfig,
{
    /// The internal config of the WindowManager.
    config: C,
    /// The desktop containing all windows.
    desktop: Desktop<P>,
    /// All screens connected to the computer.
    screens: Ring<Screen>,
    /// A main error handler function.
    ehandler: Box<dyn ErrorHandler<P, C>>,
    /// The window currently being manipulated
    /// if `self.mousemode` is not None.
    selected: Option<P::Client>,
    /// Used when window is moved to track pointer location.
    last_mouse_pos: Point,
    // If the wm is running.
    running: bool,
    // Set if the loop breaks and the user wants a restart.
    restart: bool,
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
    pub fn new<E, W, L>(mut config: E) -> Result<Toaru<P, C>, P>
    where
        E: Config<P, Runtime = C, Workspaces = W, Layouts = L>,
        W: IntoIterator<Item = WorkspaceSpec>,
        L: IntoIterator<Item = Box<dyn Layout<P>>>,
    {
        let workspaces: Vec<WorkspaceSpec> = config.take_workspaces().into_iter().collect();
        let layouts = Layouts::with_layouts_validated(
            config.take_layouts()
                .into_iter()
                .collect::<Vec<Box< dyn Layout<P>>>>()   
        )?;

        let desktop = Desktop::new(workspaces, layouts)?;

        Ok(Self {
            config: config.into_runtime_config(),
            desktop,
            screens: Ring::new(),
            ehandler: Box::new(DefaultErrorHandler),
            selected: None,
            last_mouse_pos: Point {x: 0, y: 0},
            running: false,
            restart: false,
        })
    }

    //* Public Methods

    /// Registers the executable as a window manager
    /// with the X server, as well as setting properties
    /// required by ICCCM or EWMH.
    ///
    /// Selects for subtructure redirect and notify,
    /// grabs required keys for keybinds,
    /// and runs any registered startup hooks.
    pub fn register<I>(&mut self, hooks: I)
    where
        I: IntoIterator<Item = Hook<P, C>>,
    {
        todo!()
    }

    /// Run an external command.
    pub fn run_external<S: AsRef<OsStr>>(&mut self, cmd: S, args: &[S]) {
        debug!("Running command [{:?}]", <S as AsRef<OsStr>>::as_ref(&cmd));
        let result = Command::new(&cmd)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match result {
            Ok(_) => {}
            Err(e) => (self.ehandler).call(self.state(), ToaruError::SpawnProc(e.to_string())),
        }
    }

    /// Starts an external command and maintains a handle to it.
    #[allow(unused_variables)]
    pub fn start_external<S: AsRef<OsStr>>(&mut self, cmd: S, args: &[S]) {
        todo!()
    }

    /// Provides a WMState for introspection.
    pub fn state(&self) -> ToaruState<'_, P, C> {
        ToaruState {
            config: &self.config,
            workspaces: &self.desktop.workspaces,
            desktop: &self.desktop,
            selected: self.selected.as_ref(),
        }
    }

    /// Set an error handler for WindowManager.
    pub fn set_error_handler<E>(&mut self, ehandler: E)
    where
        E: ErrorHandler<P, C> + 'static,
    {
        self.ehandler = Box::new(ehandler);
    }

    /// Quits the event loop.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Restarts the window manager in-place.
    pub fn restart(&mut self) {
        self.running = false;
        self.restart = true;
    }

    /// Dumps the internal state of WindowManager to stderr.
    pub fn dump_internal_state(&self) {
        eprintln!("============== | INTERNAL STATE DUMP | ==============");
        eprintln!("{:#?}", &self);
        eprintln!("====================| END DUMP |=====================")
    }
}

/// Desktop-level commands.
impl<P, C> Toaru<P, C>
where
    P: Platform<Error = ToaruError<P>>,
    C: RuntimeConfig,
{
    /// Goes to the specified workspace.
    #[instrument(level = "debug", skip(self))]
    pub fn goto_workspace(&mut self, name: &str) {
        // handle_err!(
        //     self.desktop.go_to(
        //         name,
        //         &self.platform,
        //         self.screens.focused().unwrap(),
        //         &self.config
        //     ),
        //     self
        // );
    }

    /// Cycles the focused workspace.
    pub fn cycle_workspace(&mut self, direction: Direction) {
        // handle_err!(
        //     self.desktop.cycle_to(
        //         &self.platform,
        //         self.screens.focused().unwrap(),
        //         &self.config,
        //         direction
        //     ),
        //     self
        // );
    }

    /// Sends the focused window to the specified workspace.
    pub fn send_focused_to(&mut self, name: &str) {
        // handle_err!(
        //     self.desktop.send_focused_to(
        //         name,
        //         &self.platform,
        //         self.screens.focused().unwrap(),
        //         &self.config
        //     ),
        //     self
        // );
    }

    /// Sends the focused window to the specified workspace and then switches to it.
    pub fn send_window_and_switch(&mut self, name: &str) {
        // handle_err!(
        //     self.desktop.send_focused_to(
        //         name,
        //         &self.platform,
        //         self.screens.focused().unwrap(),
        //         &self.config
        //     ),
        //     self
        // );
        // handle_err!(
        //     self.desktop.go_to(
        //         name,
        //         &self.platform,
        //         self.screens.focused().unwrap(),
        //         &self.config
        //     ),
        //     self
        // );
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
        // self.desktop
        //     .current_mut()
        //     .cycle_focus(direction, &self.platform.handle(), &self.config);
    }

    /// Cycles in the given direction to the layout applied to the current workspace.
    pub fn cycle_layout(&mut self, direction: Direction) {
        // self.desktop.current_mut().cycle_layout(
        //     direction,
        //     &self.platform,
        //     self.screens.focused().unwrap(),
        //     &self.config,
        // )
    }

    /// Toggles the state of the focused window to floating or vice versa.
    pub fn toggle_focused_state(&mut self) {
        // self.desktop.current_mut().toggle_focused_state(
        //     &self.platform,
        //     self.screens.focused().unwrap(),
        //     &self.config,
        // );
    }

    /// Sends an [`Update`](crate::layouts::update::Update)
    /// to the current layout.
    pub fn update_current_layout<U: IntoUpdate>(&mut self, update: U) {
        // self.desktop.current_mut().update_focused_layout(
        //     update,
        //     &self.platform,
        //     self.screens.focused().unwrap(),
        //     &self.config,
        // )
    }

    /// Switches to the given layout on the current workspace.
    pub fn switch_layout<S: AsRef<str>>(&mut self, name: S) {
        // self.desktop.current_mut().switch_layout(
        //     name,
        //     &self.platform,
        //     self.screens.focused().unwrap(),
        //     &self.config,
        // )
    }

    /// Toggles the focused window to fullscreen.
    pub fn toggle_focused_fullscreen(&mut self) {
        // self.desktop.current_mut().toggle_focused_fullscreen(
        //     &self.platform,
        //     self.screens.focused().unwrap(),
        //     &self.config,
        // );
    }

    /// Grabs the pointer and moves the window the pointer is on.
    ///
    /// If the selected window is under layout, it is removed from
    /// layout and the entire workspace is then re-laid out.
    //#[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    pub fn move_window_ptr(&mut self, pt: Point) {
        // let (dx, dy) = self.last_mouse_pos.calculate_offset(pt);

        // if let Some(win) = self.selected {
        //     _rm_if_under_layout!(self, win);

        //     let current = self.desktop.current_mut();
        //     if let Some(win) = current.windows.lookup_mut(win) {
        //         win.do_move(&self.platform, dx, dy);
        //     } else {
        //         error!("Tried to move untracked window {}", win)
        //     }
        // } else {
        //     warn!("no selected window to move");
        // }

        // self.last_mouse_pos = pt;
    }

    /// Grabs the pointer and resizes the window the pointer is on.
    ///
    /// If the selected window is under layout, it is removed from
    /// layout and the entire workspace is then re-laid out.
    //#[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    pub fn resize_window_ptr(&mut self, pt: Point) {
        // let (dx, dy) = self.last_mouse_pos.calculate_offset(pt);

        // if let Some(win) = self.selected {
        //     _rm_if_under_layout!(self, win);

        //     let current = self.desktop.current_mut();
        //     if let Some(win) = current.windows.lookup_mut(win) {
        //         win.do_resize(&self.platform, dx, dy);
        //     } else {
        //         error!("Tried to move untracked window {}", win)
        //     }
        // } else {
        //     warn!("no selected window to resize");
        // }

        // self.last_mouse_pos = pt;
    }

    /// Moves the window `delta` pixels in direction `dir`.
    pub fn move_window(&mut self, delta: i32, dir: Cardinal) {
        // if let Some(id) = self.focused_client_id() {
        //     _rm_if_under_layout!(self, id);
        // }

        // let current = self.desktop.current_mut();
        // if let Some(win) = current.focused_client_mut() {
        //     match dir {
        //         Cardinal::Up => win.do_move(&self.platform, 0, -delta),
        //         Cardinal::Down => win.do_move(&self.platform, 0, delta),
        //         Cardinal::Left => win.do_move(&self.platform, -delta, 0),
        //         Cardinal::Right => win.do_move(&self.platform, delta, 0),
        //     }
        // }
    }

    /// Resizes the window `delta` pixels in direction `dir`.
    pub fn resize_window(&mut self, delta: i32, dir: Cardinal) {
        // if let Some(id) = self.focused_client_id() {
        //     _rm_if_under_layout!(self, id);
        // }

        // let current = self.desktop.current_mut();
        // if let Some(win) = current.focused_client_mut() {
        //     match dir {
        //         Cardinal::Up => win.do_resize(&self.platform, 0, -delta),
        //         Cardinal::Down => win.do_resize(&self.platform, 0, delta),
        //         Cardinal::Left => win.do_resize(&self.platform, -delta, 0),
        //         Cardinal::Right => win.do_resize(&self.platform, delta, 0),
        //     }
        // }
    }

    /// Closes the focused window.
    pub fn close_focused_window(&mut self) {
        // if let Some(window) = self.desktop.current_mut().windows.focused() {
        //     handle_err!(self.platform.destroy_window(window.id()), self);
        // } else {
        //     warn!("Could not find focused window to destroy");
        // }
    }
}

#[doc(hidden)]
//* Private Methods *//
impl<P, C> Toaru<P, C>
where
    P: Platform,
    C: RuntimeConfig,
{
    /// Receive the next event from the connection and process it
    /// into a actions to be taken by the window manager.
    fn process_next_event(&mut self) -> Result<Option<Vec<EventAction<P>>>, P> {
        // let Some(event) = self.platform.poll_next_event()? else {return Ok(None)};
        // Ok(EventAction::from_xevent(event, self.state()))
        todo!()
    }

    #[cfg_attr(
        debug_assertions,
        instrument(level = "debug", skip(self, actions, mousebinds, keybinds))
    )]
    fn handle_event(
        &mut self,
        actions: Vec<EventAction<P>>,
        mousebinds: &mut Mousebinds<P, C>,
        keybinds: &mut Keybinds<P, C>,
    ) -> Result<(), P> {

        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn update_focus(&mut self, id: &P::Client) -> Result<(), P> {
        // get target id
        // set input focus to main window
        // send clientmessage to focus if not focused
        // set focused border colour
        // set unfocused border colour
        // update focus internally
        // if client not found, set focus to root window
        // let target = if self.desktop.is_managing(id) {
        //     id
        // } else {
        //     match self.focused_client_id() {
        //         Some(c) => c,
        //         None =>
        //         /*handle this*/
        //         {
        //             return Err(ToaruError::UnknownClient(id))
        //         }
        //     }
        // };
        // self.desktop
        //     .current_mut()
        //     .focus_window(target, &self.conn, &self.config);
        Ok(())
    }

    fn focused_client_id(&self) -> Option<&P::Client> {
        self.desktop.current_client().map(|c| c.id())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn set_focused_screen(&mut self, ptr: Option<Point>) -> Result<(), P> {
        
        //todo: if per-screen workspaces, need to focus workspace also
        Ok(())
    }

    /// Query _NET_WM_NAME or WM_NAME and change it accordingly
    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn client_name_change(&mut self, id: &P::Client) -> Result<(), P> {
        //todo
        // if let Some(c) = self.desktop.current_client_mut() {
        //     c.update_dynamic(&self.conn, &self.config);
        // }
        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn map_tracked_client(&mut self, id: &P::Client) -> Result<(), P> {
        // let current = self.desktop.current_mut();
        // if self.conn.should_float(id, self.config.float_classes()) || current.is_floating() {
        //     current.add_window_off_layout(
        //         id,
        //         &self.conn,
        //         self.screens.focused().unwrap(),
        //         &self.config,
        //     )
        // } else {
        //     current.add_window_on_layout(
        //         id,
        //         &self.conn,
        //         self.screens.focused().unwrap(),
        //         &self.config,
        //     )
        // }
        Ok(())
    }

    fn map_untracked_client(&self, id: &P::Client) -> Result<(), P> {
        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn unmap_client(&mut self, id: &P::Client) -> Result<(), P> {
        // the client itself handles the unmapping, so we just handle internal state
        // self.desktop.current_mut().del_window(
        //     id,
        //     &self.conn,
        //     self.screens.focused().unwrap(),
        //     &self.config,
        // )?;
        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn configure_client(&mut self, /* data: ConfigureRequestData */ ) -> Result<(), P> {
        //todo
        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn client_to_workspace(&mut self, id: &P::Client, idx: usize) -> Result<(), P> {
        // let name = match self.desktop.get(idx) {
        //     Some(ws) => ws.name.to_string(),
        //     None => return Ok(()),
        // };

        // self.desktop.send_window_to(
        //     id,
        //     &name,
        //     &self.conn,
        //     self.screens.focused().unwrap(),
        //     &self.config,
        // )

        Ok(())
    }

    /// Runs the keybind.
    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self, bdgs)))]
    fn run_keybind(&mut self, kb: Keybind, bdgs: &mut Keybinds<P, C>, id: &P::Client) {
        if let Some(cb) = bdgs.get_mut(&kb) {
            cb(self);
        } else {
            warn!("Binding not found for keypress event");
        }
    }

    //#[cfg_attr(debug_assertions, instrument(level = "debug", skip(self, bdgs)))]
    fn run_mousebind(
        &mut self,
        mb: Mousebind,
        bdgs: &mut Mousebinds<P, C>,
        id: &P::Client,
        pt: Point,
    ) -> Result<(), P> {
        // match mb.kind {
        //     // assume that we want to do something with the pointer,
        //     // so grab it
        //     MouseEventKind::Press => {
        //         self.platform.grab_pointer(self.root.id, 0)?;
        //         self.selected = Some(id);
        //         self.last_mouse_pos = pt;
        //     }
        //     MouseEventKind::Release => {
        //         self.platform.ungrab_pointer()?;
        //         self.selected = None;
        //         self.last_mouse_pos = pt;
        //     }
        //     MouseEventKind::Motion => {}
        // }

        // if let Some(cb) = bdgs.get_mut(&mb) {
        //     cb(self, pt);
        // } else {
        //     warn!("Binding not found for mouse event");
        // }
        Ok(())
    }

    fn set_fullscreen(&mut self, _id: &P::Client, _should_fullscreen: bool) -> Result<(), P> {
        todo!()
    }

    fn toggle_urgency(&mut self, _id: &P::Client) -> Result<(), P> {
        //todo
        Ok(())
    }

    fn screen_reconfigure(&mut self) -> Result<(), P> {
        todo!()
    }

    fn focus_screen(&mut self, idx: usize) {
        self.screens.set_focused(idx);
    }

    pub(crate) fn handle_error(&mut self, err: P::Error, /* _evt: XEvent */) {
        // (self.ehandler).call(self.state(), ToaruError::BackendError(err.into()));
    }
}

impl<P, C> fmt::Debug for Toaru<P, C>
where
    P: Platform,
    C: RuntimeConfig,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WindowManager")
            //.field("config", &self.config)
            .field("workspaces", &self.desktop.workspaces)
            .field("screens", &self.screens)
            //.field("root", &self.root)
            .field("selected", &self.selected)
            .finish()
    }
}
