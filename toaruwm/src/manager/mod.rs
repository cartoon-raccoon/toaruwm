//! The window manager itself, and associated modules.

//#![allow(unused_variables, unused_imports, dead_code)]
use std::ffi::OsStr;
use std::fmt;
use std::iter::FromIterator;
use std::process::{Command, Stdio};

//use std::marker::PhantomData;

//use std::sync::OnceLock;

use tracing::instrument;
use tracing::{debug, error, info, span, warn, Level};

use crate::bindings::{Keybind, Keybinds, Mousebind, Mousebinds};
use crate::core::{Desktop, Screen, WorkspaceSpec};
use crate::layouts::{update::IntoUpdate, Layout, Layouts};
use crate::log::DefaultErrorHandler;
use crate::types::{Cardinal, ClientAttrs, Direction, Point, Ring, Selector};
use crate::x::{
    event::ConfigureRequestData, input::MouseEventKind, Atom, Property, XConn, XError, XEvent,
    XWindow, XWindowID,
};
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
pub use state::{RuntimeConfig, WmState};

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
                &$_self.conn,
                $id,
                $_self.screens.focused().unwrap(),
                &$_self.config,
            );
        }
    };
}

/// The main window manager object that owns the event loop,
/// and receives and responds to events.
///
/// `WindowManager` is generic over two types:
///
/// - X, that is its connection to the X server and so must implement
/// the [`XConn`] trait. This is the type by which the `WindowManager`
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
/// A WindowManager combines an [`XConn`] and a [`Desktop`] which
/// combines [`Workspace`][1]s. As the top-level struct, `WindowManager`
/// has methods that apply to its sub-structures, and thus are
/// organized accordingly: `WindowManager`-level, `Desktop`-level,
/// and `Workspace`-level.
///
/// # Usage
///
/// To run a WindowManager, it needs to first be registered with
/// the X server, so as to select the event masks on the root window
/// required for its operation. It then needs to grab user keybinds
/// and mousebindings and register those with the X server as well.
/// After that, then it can initiate the event loop.
///
/// ```no_run
/// use toaruwm::{XCBConn, ToaruConfig, WindowManager};
/// use toaruwm::bindings::{Keybinds, Mousebinds};
///
/// let conn = XCBConn::new().unwrap();
///
/// let mut wm = WindowManager::new(conn, ToaruConfig::default())
///     .expect("could not create WindowManager");
///
/// /* register the windowmanager with the x server */
/// wm.register(Vec::new());
///
/// /* run the windowmanager, ideally grabbing your keybinds first! */
/// wm.run(Keybinds::new(), Mousebinds::new()).unwrap();
/// ```
///
/// The WindowManager has a few methods defined on it that allow you
/// to control its behaviour. These methods are usually invoked through
/// a callback triggered by a keybind or mousebind.
///
/// ## Example
///
/// ```rust
/// # use toaruwm::{ToaruWM, InitXCB};
/// use toaruwm::bindings::{ModKey, Keybind, Keybinds, Mousebinds};
///
/// /* create a new keybinds object */
/// let mut keybinds = Keybinds::new();
///
/// /* create a binding */
/// let kb = Keybind::new(vec![ModKey::Meta], 240); //todo: use keybinds macro
///
/// /* set a callback to run (with type annotations for clarity) */
/// keybinds.insert(kb, |wm: &mut ToaruWM<InitXCB>| {
///     wm.run_external("xterm", &[])
/// });
/// ```
///
/// # Configuration
///
/// A `WindowManager` can be configured with any type implementing
/// the [`Config`] trait.
///
/// # Defaults
///
/// The Window Manager employs a basic error handler that simply logs
/// errors to stdout, but can be changed with
/// `WindowManager::set_error_handler`.
///
/// [1]: crate::core::Workspace
pub struct WindowManager<X, C>
where
    X: XConn,
    C: RuntimeConfig,
{
    /// The X Connection
    conn: X,
    /// The internal config of the WindowManager.
    config: C,
    /// The desktop containing all windows.
    desktop: Desktop,
    /// All screens connected to the computer.
    screens: Ring<Screen>,
    /// The root window.
    root: XWindow,
    /// A main error handler function.
    ehandler: Box<dyn ErrorHandler<X, C>>,
    /// The window currently being manipulated
    /// if `self.mousemode` is not None.
    selected: Option<XWindowID>,
    /// Used when window is moved to track pointer location.
    last_mouse_pos: Point,
    // If the wm is running.
    running: bool,
    // Set if the loop breaks and the user wants a restart.
    restart: bool,
}

/// General `WindowManager`-level commands.
impl<X, C> WindowManager<X, C>
where
    X: XConn,
    C: RuntimeConfig,
{
    /// Constructs a new WindowManager object.
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
    pub fn new<E, W, L>(conn: X, mut config: E) -> Result<WindowManager<X, C>>
    where
        E: Config<Runtime = C, Workspaces = W, Layouts = L>,
        W: IntoIterator<Item = WorkspaceSpec>,
        L: IntoIterator<Item = Box<dyn Layout>>,
    {
        let root = conn.get_root();
        let mut screens = Ring::from_iter(
            conn.all_outputs()
                .unwrap_or_else(|e| fatal!("Could not get screens: {}", e)),
        );
        let workspaces: Vec<WorkspaceSpec> = config.take_workspaces().into_iter().collect();

        let layouts = Layouts::with_layouts_validated(
            config
                .take_layouts()
                .into_iter()
                .collect::<Vec<Box<dyn Layout>>>(),
        )?;
        info!(target: "", "Layouts successfully validated");

        let mut just_workspaces = Vec::with_capacity(workspaces.len());

        for spec in workspaces {
            if let Some(scr) = screens.get_mut(spec.idx) {
                scr.add_workspace(&spec.name);
                just_workspaces.push(spec);
            } else {
                error!("No screen with index {}", spec.idx);
            }
        }

        debug!("Got screens: {:?}", screens);
        screens.set_focused(0);

        Ok(Self {
            conn,
            config: config.into_runtime_config(),
            desktop: Desktop::new(just_workspaces, layouts)?,
            screens,
            root,
            ehandler: Box::new(DefaultErrorHandler),
            selected: None,
            //focused: None,
            last_mouse_pos: Point { x: 0, y: 0 },
            running: true,
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
        I: IntoIterator<Item = Hook<X, C>>,
    {
        info!("Registering window manager");

        let root = self.conn.get_root();

        debug!("Got root window data: {:#?}", root);

        self.conn
            .change_window_attributes(root.id, &[ClientAttrs::RootEventMask])
            .unwrap_or_else(|_| {
                error!("Another window manager is running.");
                std::process::exit(1)
            });

        // set supported protocols
        debug!("Setting supported protocols");
        self.conn
            .set_supported(&[
                Atom::WmProtocols,
                Atom::WmTakeFocus,
                Atom::WmState,
                Atom::WmDeleteWindow,
            ])
            .unwrap_or_else(|e| {
                error!("{}", e);
                std::process::exit(1)
            });

        // set _NET_NUMBER_OF_DESKTOPS
        debug!("Setting _NET_NUMBER_OF_DESKTOPS");
        self.conn
            .set_property(
                root.id,
                Atom::NetNumberOfDesktops.as_ref(),
                Property::Cardinal(self.desktop.workspaces.len() as u32),
            )
            .unwrap_or_else(|e| {
                error!("{}", e);
            });

        // set _NET_CURRENT_DESKTOP
        self.conn
            .set_property(
                root.id,
                Atom::NetCurrentDesktop.as_ref(),
                Property::Cardinal(0),
            )
            .unwrap_or_else(|e| {
                error!("{}", e);
                std::process::exit(1)
            });

        // run hooks
        for mut hook in hooks {
            hook(self);
        }
    }

    /// Grabs bindings and runs the window manager.
    pub fn grab_and_run(&mut self, kb: Keybinds<X, C>, mb: Mousebinds<X, C>) -> Result<()> {
        self.grab_bindings(&kb, &mb)?;
        self.run(kb, mb)
    }

    /// Grabs the given key and mouse bindings.
    pub fn grab_bindings(&mut self, kb: &Keybinds<X, C>, mb: &Mousebinds<X, C>) -> Result<()> {
        info!(target: "", "Grabbing mouse bindings");
        let root_id = self.conn.get_root().id;
        for binding in mb.keys() {
            self.conn.grab_button(*binding, root_id, true)?;
        }

        info!(target: "", "Grabbing key bindings");
        for binding in kb.keys() {
            self.conn.grab_key(*binding, root_id)?;
        }

        Ok(())
    }

    /// Runs the main event loop.
    pub fn run(&mut self, mut kb: Keybinds<X, C>, mut mb: Mousebinds<X, C>) -> Result<()> {
        // grab all existing windows
        info!(target: "", "Grabbing any existing windows");
        for _ in self.conn.query_tree(self.root.id)? {
            //todo
        }

        info!(target: "", "Setup complete, beginning event loop");
        loop {
            // mark the start of an event loop
            let span = span!(Level::DEBUG, "evloop");
            debug!("================== Event Loop Start ==================");
            let enter = span.enter();

            let event = self.process_next_event().or_else(|e| {
                match e {
                    // only return if the error is a connection error
                    ToaruError::XConnError(XError::Connection(_)) => Err(e),
                    // else, handle error and map to an Ok(None)
                    e => {
                        (self.ehandler).call(self.state(), e);
                        Ok(None)
                    }
                }
            })?;
            trace!("received events {:#?}", event);
            if let Some(actions) = event {
                // if event handling returned an error, do not return
                // instead, handle it internally and continue
                handle_err!(self.handle_event(actions, &mut mb, &mut kb), self);
            }

            //* update window properties

            if !self.running {
                break;
            }

            // mark the end of an event loop iteration
            drop(enter);
        }

        if self.restart {
            todo!("restart process")
        }

        Ok(())
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
    pub fn state(&self) -> WmState<'_, X, C> {
        WmState {
            conn: &self.conn,
            config: &self.config,
            workspaces: &self.desktop.workspaces,
            desktop: &self.desktop,
            root: self.root,
            selected: self.selected,
        }
    }

    /// Set an error handler for WindowManager.
    pub fn set_error_handler<E>(&mut self, ehandler: E)
    where
        E: ErrorHandler<X, C> + 'static,
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
impl<X, C> WindowManager<X, C>
where
    X: XConn,
    C: RuntimeConfig,
{
    /// Goes to the specified workspace.
    #[instrument(level = "debug", skip(self))]
    pub fn goto_workspace(&mut self, name: &str) {
        handle_err!(
            self.desktop.go_to(
                name,
                &self.conn,
                self.screens.focused().unwrap(),
                &self.config
            ),
            self
        );
    }

    /// Cycles the focused workspace.
    pub fn cycle_workspace(&mut self, direction: Direction) {
        handle_err!(
            self.desktop.cycle_to(
                &self.conn,
                self.screens.focused().unwrap(),
                &self.config,
                direction
            ),
            self
        );
    }

    /// Sends the focused window to the specified workspace.
    pub fn send_focused_to(&mut self, name: &str) {
        handle_err!(
            self.desktop.send_focused_to(
                name,
                &self.conn,
                self.screens.focused().unwrap(),
                &self.config
            ),
            self
        );
    }

    /// Sends the focused window to the specified workspace and then switches to it.
    pub fn send_window_and_switch(&mut self, name: &str) {
        handle_err!(
            self.desktop.send_focused_to(
                name,
                &self.conn,
                self.screens.focused().unwrap(),
                &self.config
            ),
            self
        );
        handle_err!(
            self.desktop.go_to(
                name,
                &self.conn,
                self.screens.focused().unwrap(),
                &self.config
            ),
            self
        );
    }
}

/// Workspace-level commands.
impl<X, C> WindowManager<X, C>
where
    X: XConn,
    C: RuntimeConfig,
{
    /// Cycles the focused window.
    pub fn cycle_focus(&mut self, direction: Direction) {
        self.desktop
            .current_mut()
            .cycle_focus(direction, &self.conn, &self.config);
    }

    /// Cycles in the given direction to the layout applied to the current workspace.
    pub fn cycle_layout(&mut self, direction: Direction) {
        self.desktop.current_mut().cycle_layout(
            direction,
            &self.conn,
            self.screens.focused().unwrap(),
            &self.config,
        )
    }

    /// Toggles the state of the focused window to floating or vice versa.
    pub fn toggle_focused_state(&mut self) {
        self.desktop.current_mut().toggle_focused_state(
            &self.conn,
            self.screens.focused().unwrap(),
            &self.config,
        );
    }

    /// Sends an [`Update`](crate::layouts::update::Update)
    /// to the current layout.
    pub fn update_current_layout<U: IntoUpdate>(&mut self, update: U) {
        self.desktop.current_mut().update_focused_layout(
            update,
            &self.conn,
            self.screens.focused().unwrap(),
            &self.config,
        )
    }

    /// Switches to the given layout on the current workspace.
    pub fn switch_layout<S: AsRef<str>>(&mut self, name: S) {
        self.desktop.current_mut().switch_layout(
            name,
            &self.conn,
            self.screens.focused().unwrap(),
            &self.config,
        )
    }

    /// Toggles the focused window to fullscreen.
    pub fn toggle_focused_fullscreen(&mut self) {
        self.desktop.current_mut().toggle_focused_fullscreen(
            &self.conn,
            self.screens.focused().unwrap(),
            &self.config,
        );
    }

    /// Grabs the pointer and moves the window the pointer is on.
    ///
    /// If the selected window is under layout, it is removed from
    /// layout and the entire workspace is then re-laid out.
    //#[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    pub fn move_window_ptr(&mut self, pt: Point) {
        let (dx, dy) = self.last_mouse_pos.calculate_offset(pt);

        if let Some(win) = self.selected {
            _rm_if_under_layout!(self, win);

            let current = self.desktop.current_mut();
            if let Some(win) = current.windows.lookup_mut(win) {
                win.do_move(&self.conn, dx, dy);
            } else {
                error!("Tried to move untracked window {}", win)
            }
        } else {
            warn!("no selected window to move");
        }

        self.last_mouse_pos = pt;
    }

    /// Grabs the pointer and resizes the window the pointer is on.
    ///
    /// If the selected window is under layout, it is removed from
    /// layout and the entire workspace is then re-laid out.
    //#[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    pub fn resize_window_ptr(&mut self, pt: Point) {
        let (dx, dy) = self.last_mouse_pos.calculate_offset(pt);

        if let Some(win) = self.selected {
            _rm_if_under_layout!(self, win);

            let current = self.desktop.current_mut();
            if let Some(win) = current.windows.lookup_mut(win) {
                win.do_resize(&self.conn, dx, dy);
            } else {
                error!("Tried to move untracked window {}", win)
            }
        } else {
            warn!("no selected window to resize");
        }

        self.last_mouse_pos = pt;
    }

    /// Moves the window `delta` pixels in direction `dir`.
    pub fn move_window(&mut self, delta: i32, dir: Cardinal) {
        if let Some(id) = self.focused_client_id() {
            _rm_if_under_layout!(self, id);
        }

        let current = self.desktop.current_mut();
        if let Some(win) = current.focused_client_mut() {
            match dir {
                Cardinal::Up => win.do_move(&self.conn, 0, -delta),
                Cardinal::Down => win.do_move(&self.conn, 0, delta),
                Cardinal::Left => win.do_move(&self.conn, -delta, 0),
                Cardinal::Right => win.do_move(&self.conn, delta, 0),
            }
        }
    }

    /// Resizes the window `delta` pixels in direction `dir`.
    pub fn resize_window(&mut self, delta: i32, dir: Cardinal) {
        if let Some(id) = self.focused_client_id() {
            _rm_if_under_layout!(self, id);
        }

        let current = self.desktop.current_mut();
        if let Some(win) = current.focused_client_mut() {
            match dir {
                Cardinal::Up => win.do_resize(&self.conn, 0, -delta),
                Cardinal::Down => win.do_resize(&self.conn, 0, delta),
                Cardinal::Left => win.do_resize(&self.conn, -delta, 0),
                Cardinal::Right => win.do_resize(&self.conn, delta, 0),
            }
        }
    }

    /// Closes the focused window.
    pub fn close_focused_window(&mut self) {
        if let Some(window) = self.desktop.current_mut().windows.focused() {
            handle_err!(self.conn.destroy_window(window.id()), self);
        } else {
            warn!("Could not find focused window to destroy");
        }
    }
}

#[doc(hidden)]
//* Private Methods *//
impl<X, C> WindowManager<X, C>
where
    X: XConn,
    C: RuntimeConfig,
{
    /// Receive the next event from the connection and process it
    /// into a actions to be taken by the window manager.
    fn process_next_event(&mut self) -> Result<Option<Vec<EventAction>>> {
        let Some(event) = self.conn.poll_next_event()? else {return Ok(None)};
        Ok(EventAction::from_xevent(event, self.state()))
    }

    #[cfg_attr(
        debug_assertions,
        instrument(level = "debug", skip(self, actions, mousebinds, keybinds))
    )]
    fn handle_event(
        &mut self,
        actions: Vec<EventAction>,
        mousebinds: &mut Mousebinds<X, C>,
        keybinds: &mut Keybinds<X, C>,
    ) -> Result<()> {
        use EventAction::*;

        for action in actions {
            match action {
                MoveClientFocus(id) => self.update_focus(id)?,
                ClientNameChange(id) => self.client_name_change(id)?,
                ScreenReconfigure => self.screen_reconfigure()?,
                SetFocusedScreen(pt) => self.set_focused_screen(pt)?,
                DestroyClient(_) => {}
                MapTrackedClient(id) => self.map_tracked_client(id)?,
                MapUntrackedClient(id) => self.map_untracked_client(id)?,
                UnmapClient(id) => self.unmap_client(id)?,
                ConfigureClient(data) => self.configure_client(data)?,
                ClientToWorkspace(id, idx) => self.client_to_workspace(id, idx)?,
                RunKeybind(kb, id) => self.run_keybind(kb, keybinds, id),
                RunMousebind(mb, id, pt) => self.run_mousebind(mb, mousebinds, id, pt)?,
                ToggleClientFullscreen(id, should_fs) => self.set_fullscreen(id, should_fs)?,
                ToggleUrgency(id) => self.toggle_urgency(id)?,
                HandleError(err, evt) => self.handle_error(err, evt),
            }
        }

        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn update_focus(&mut self, id: XWindowID) -> Result<()> {
        // get target id
        // set input focus to main window
        // send clientmessage to focus if not focused
        // set focused border colour
        // set unfocused border colour
        // update focus internally
        // if client not found, set focus to root window
        let target = if self.desktop.is_managing(id) {
            id
        } else {
            match self.focused_client_id() {
                Some(c) => c,
                None =>
                /*handle this*/
                {
                    return Err(ToaruError::UnknownClient(id))
                }
            }
        };
        self.desktop
            .current_mut()
            .focus_window(target, &self.conn, &self.config);
        Ok(())
    }

    fn focused_client_id(&self) -> Option<XWindowID> {
        self.desktop.current_client().map(|c| c.id())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn set_focused_screen(&mut self, ptr: Option<Point>) -> Result<()> {
        // get pointer position
        // if ptr is None, query the pointer directly
        let ptr = if let Some(ptr) = ptr {
            ptr
        } else {
            let pq = self.conn.query_pointer(self.root.id)?;
            Point::new(pq.root_x, pq.root_y)
        };

        // get the screen to focus to
        let to_focus = self.screens.index(Selector::Condition(&|s| {
            s.effective_geom().contains_point(ptr)
        }));

        if let Some(idx) = to_focus {
            self.focus_screen(idx);
        } else {
            return Err(ToaruError::InvalidPoint(ptr.x, ptr.y));
        }
        //todo: if per-screen workspaces, need to focus workspace also
        Ok(())
    }

    /// Query _NET_WM_NAME or WM_NAME and change it accordingly
    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn client_name_change(&mut self, id: XWindowID) -> Result<()> {
        //todo
        if let Some(c) = self.desktop.current_client_mut() {
            c.update_dynamic(&self.conn, &self.config);
        }
        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn map_tracked_client(&mut self, id: XWindowID) -> Result<()> {
        let current = self.desktop.current_mut();
        if self.conn.should_float(id, self.config.float_classes()) || current.is_floating() {
            current.add_window_off_layout(
                id,
                &self.conn,
                self.screens.focused().unwrap(),
                &self.config,
            )
        } else {
            current.add_window_on_layout(
                id,
                &self.conn,
                self.screens.focused().unwrap(),
                &self.config,
            )
        }
        Ok(())
    }

    fn map_untracked_client(&self, id: XWindowID) -> Result<()> {
        Ok(self.conn.map_window(id)?)
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn unmap_client(&mut self, id: XWindowID) -> Result<()> {
        // the client itself handles the unmapping, so we just handle internal state
        self.desktop.current_mut().del_window(
            id,
            &self.conn,
            self.screens.focused().unwrap(),
            &self.config,
        )?;
        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn configure_client(&mut self, data: ConfigureRequestData) -> Result<()> {
        //todo
        Ok(())
    }

    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self)))]
    fn client_to_workspace(&mut self, id: XWindowID, idx: usize) -> Result<()> {
        let name = match self.desktop.get(idx) {
            Some(ws) => ws.name.to_string(),
            None => return Ok(()),
        };

        self.desktop.send_window_to(
            id,
            &name,
            &self.conn,
            self.screens.focused().unwrap(),
            &self.config,
        )
    }

    /// Runs the keybind.
    #[cfg_attr(debug_assertions, instrument(level = "debug", skip(self, bdgs)))]
    fn run_keybind(&mut self, kb: Keybind, bdgs: &mut Keybinds<X, C>, id: XWindowID) {
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
        bdgs: &mut Mousebinds<X, C>,
        id: XWindowID,
        pt: Point,
    ) -> Result<()> {
        match mb.kind {
            // assume that we want to do something with the pointer,
            // so grab it
            MouseEventKind::Press => {
                self.conn.grab_pointer(self.root.id, 0)?;
                self.selected = Some(id);
                self.last_mouse_pos = pt;
            }
            MouseEventKind::Release => {
                self.conn.ungrab_pointer()?;
                self.selected = None;
                self.last_mouse_pos = pt;
            }
            MouseEventKind::Motion => {}
        }

        if let Some(cb) = bdgs.get_mut(&mb) {
            cb(self, pt);
        } else {
            warn!("Binding not found for mouse event");
        }
        Ok(())
    }

    fn set_fullscreen(&mut self, _id: XWindowID, _should_fullscreen: bool) -> Result<()> {
        todo!()
    }

    fn toggle_urgency(&mut self, _id: XWindowID) -> Result<()> {
        //todo
        Ok(())
    }

    fn screen_reconfigure(&mut self) -> Result<()> {
        todo!()
    }

    fn focus_screen(&mut self, idx: usize) {
        self.screens.set_focused(idx);
    }

    fn handle_error(&mut self, err: XError, _evt: XEvent) {
        (self.ehandler).call(self.state(), ToaruError::XConnError(err));
    }
}

impl<X, C> fmt::Debug for WindowManager<X, C>
where
    X: XConn,
    C: RuntimeConfig,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WindowManager")
            //.field("config", &self.config)
            .field("workspaces", &self.desktop.workspaces)
            .field("screens", &self.screens)
            .field("root", &self.root)
            .field("selected", &self.selected)
            .finish()
    }
}
