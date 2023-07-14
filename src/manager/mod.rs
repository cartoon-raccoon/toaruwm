//! The window manager itself, and associated modules.

//#![allow(unused_variables, unused_imports, dead_code)]
use std::ffi::OsStr;
use std::fmt;
use std::iter::FromIterator;
use std::process::{Command, Stdio};

//use std::marker::PhantomData;

//use std::sync::OnceLock;

use tracing::instrument;
use tracing::{debug, error, info, span, trace, warn, Level};

use crate::core::{Desktop, Screen};
use crate::keybinds::{Keybind, Keybinds, Mousebind, Mousebinds};
use crate::layouts::LayoutType;
use crate::log::DefaultErrorHandler;
use crate::types::{Cardinal, ClientAttrs, Direction, Point, Ring, Selector};
use crate::x::{
    event::ConfigureRequestData, input::MouseEventKind, Atom, Property, XConn, XError, XEvent,
    XWindow, XWindowID,
};
use crate::{ErrorHandler, Result, ToaruError};

/// The central configuration object for the window manager.
pub mod config;
/// A translation layer for converting X events into `WindowManager` actions.
pub mod event;
/// Macros and storage types for window manager hooks.
pub mod hooks;
/// Types for introspection into the WindowManager's state.
pub mod state;

#[doc(inline)]
pub use config::Config;
#[doc(inline)]
pub use event::EventAction;
#[doc(inline)]
pub use hooks::{Hook, Hooks};
#[doc(inline)]
pub use state::WmState;

//static ERR_HANDLER: OnceLock<&dyn FnMut(ToaruError)> = OnceLock::new();

macro_rules! handle_err {
    ($call:expr, $_self:expr) => {
        if let Err(e) = $call {
            $_self.ehandler.call($_self.state(), e.into());
        }
    };
}

/// The main window manager object that owns the event loop,
/// and receives and responds to events.
///
/// The manager is generic over a type argument X that
/// implements the `XConn` trait, but this is never directly exposed
/// by `WindowManager`'s public API and is only used when constructing
/// a new `WindowManager` instance.
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
/// use toaruwm::{XCBConn, Config, WindowManager};
/// use toaruwm::keybinds::{Keybinds, Mousebinds};
///
/// let conn = XCBConn::new().unwrap();
///
/// let mut wm = WindowManager::new(conn, Config::default());
///
/// /* register the windowmanager with the x server */
/// wm.register(Vec::new());
///
/// /* run the windowmanager, ideally grabbing your keybinds first! */
/// wm.run(Keybinds::new(), Mousebinds::new());
/// ```
///
/// The WindowManager has a few methods defined on it that allow you
/// to control its behaviour. These methods are usually invoked through
/// a callback triggered by a keybind or mousebind.
///
/// ## Example
///
/// ```ignore
/// use toaruwm::keybinds::{ModKey, Keybind, Keybinds, Mousebinds};
///
/// /* create a new keybinds object */
/// let mut keybinds = Keybinds::new();
///
/// /* create a binding */
/// let kb = Keybind::new(vec![ModKey::Meta], "t") // <-- use a keycode here!
///
/// /* set a callback to run */
/// keybinds.insert(|wm| {wm.run_external("xterm", &[])});
/// ```
///
/// # Defaults
///
/// The Window Manager uses a default configuration if none is provided.
/// It also employs a basic error handler that simply logs errors to
/// stdout, but can be changed with `WindowManager::set_error_handler`.
pub struct WindowManager<X: XConn> {
    /// The X Connection
    conn: X,
    /// The WM configuration.
    config: Config,
    /// The desktop containing all windows.
    desktop: Desktop,
    /// All screens connected to the computer.
    screens: Ring<Screen>,
    /// The root window.
    root: XWindow,
    /// A main error handler function.
    ehandler: Box<dyn ErrorHandler<X>>,
    /// The window currently being manipulated
    /// if `self.mousemode` is not None.
    selected: Option<XWindowID>,
    /// The window currently in focus.
    focused: Option<XWindowID>,
    /// Used when window is moved to track pointer location.
    last_mouse_pos: Point,
    // If the wm is running.
    running: bool,
    // Set if the loop breaks and the user wants a restart.
    restart: bool,
}

impl<X: XConn> fmt::Debug for WindowManager<X> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WindowManager")
            .field("config", &self.config)
            .field("workspaces", &self.desktop.workspaces)
            .field("screens", &self.screens)
            .field("root", &self.root)
            .field("selected", &self.selected)
            .field("focused", &self.focused)
            .finish()
    }
}

impl<X: XConn> WindowManager<X> {
    /// Constructs a new WindowManager object.
    pub fn new(conn: X, config: Config) -> WindowManager<X> {
        let root = conn.get_root();
        let mut screens = Ring::from_iter(
            conn.all_outputs()
                .unwrap_or_else(|e| fatal!("Could not get screens: {}", e)),
        );
        let workspaces = config.workspaces.iter().map(|(ws, _)| ws.clone()).collect();

        for (ws_name, idx) in &config.workspaces {
            if let Some(scr) = screens.get_mut(*idx) {
                scr.add_workspace(ws_name);
            } else {
                error!("No screen with index {}", idx);
            }
        }

        debug!("Got screens: {:?}", screens);
        screens.set_focused(0);

        Self {
            conn,
            config, //todo: layouttype should be specified in config
            desktop: Desktop::new(LayoutType::DTiled, None, workspaces),
            screens,
            root,
            ehandler: Box::new(DefaultErrorHandler),
            selected: None,
            focused: None,
            last_mouse_pos: Point { x: 0, y: 0 },
            running: true,
            restart: false,
        }
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
        I: IntoIterator<Item = Hook<X>>,
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
                Property::Cardinal(self.config.workspaces.len() as u32),
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
    pub fn grab_and_run(&mut self, mb: Mousebinds<X>, kb: Keybinds<X>) -> Result<()> {
        self.grab_bindings(&mb, &kb)?;
        self.run(mb, kb)
    }

    /// Grabs the given key and mouse bindings.
    pub fn grab_bindings(&mut self, mb: &Mousebinds<X>, kb: &Keybinds<X>) -> Result<()> {
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
    pub fn run(&mut self, mut mb: Mousebinds<X>, mut kb: Keybinds<X>) -> Result<()> {
        info!(target: "", "Grabbing any existing windows");
        // todo

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

    /// Set an error handler for WindowManager.
    pub fn set_error_handler<E>(&mut self, ehandler: E)
    where
        E: ErrorHandler<X> + 'static,
    {
        self.ehandler = Box::new(ehandler);
    }

    /// Goes to the specified workspace.
    #[instrument(level = "debug")]
    pub fn goto_workspace(&mut self, name: &str) {
        handle_err!(
            self.desktop
                .goto(name, &self.conn, self.screens.focused().unwrap()),
            self
        );
    }

    /// Cycles the focused workspace.
    pub fn cycle_workspace(&mut self, direction: Direction) {
        handle_err!(
            self.desktop
                .cycle_workspace(&self.conn, self.screens.focused().unwrap(), direction),
            self
        );
    }

    /// Sends the focused window to the specified workspace.
    pub fn send_focused_to(&mut self, name: &str) {
        handle_err!(
            self.desktop
                .send_focused_to(name, &self.conn, self.screens.focused().unwrap()),
            self
        );
    }

    /// Sends the focused window to the specified workspace and then switches to it.
    pub fn send_window_and_switch(&mut self, name: &str) {
        handle_err!(
            self.desktop
                .send_focused_to(name, &self.conn, self.screens.focused().unwrap()),
            self
        );
        handle_err!(
            self.desktop
                .goto(name, &self.conn, self.screens.focused().unwrap()),
            self
        );
    }

    /// Cycles the focused window.
    pub fn cycle_focus(&mut self, direction: Direction) {
        self.desktop
            .current_mut()
            .cycle_focus(&self.conn, direction);
    }

    /// Toggles the state of the focused window to floating or vice versa.
    pub fn toggle_focused_state(&mut self) {
        self.desktop
            .current_mut()
            .toggle_focused_state(&self.conn, self.screens.focused().unwrap());
    }

    /// Toggles the focused window to fullscreen.
    pub fn toggle_focused_fullscreen(&mut self) {}

    /// Cycles the layouts used by the `WindowManager`.
    pub fn cycle_layout(&mut self) {
        todo!()
    }

    /// Grabs the pointer and moves the window the pointer is on.
    #[instrument(level = "debug", skip(self))]
    pub fn move_window_ptr(&mut self, pt: Point) {
        let (dx, dy) = self.last_mouse_pos.calculate_offset(pt);

        if let Some(win) = self.selected {
            self.desktop.current_mut().set_floating(
                &self.conn,
                win,
                self.screens.focused().unwrap(),
            );
        }

        let current = self.desktop.current_mut();

        if let Some(win) = self.selected {
            if let Some(win) = current.windows.lookup_mut(win) {
                win.do_move(&self.conn, dx, dy);
            } else {
                error!("Tried to move untracked window {}", win)
            }
        } else {
            warn!("Nothing selected");
        }

        self.last_mouse_pos = pt;
    }

    /// Grabs the pointer and resizes the window the pointer is on.
    ///
    /// If the window is tiled, its state is toggled to floating
    /// and the entire desktop is re-laid out.
    pub fn resize_window_ptr(&mut self, pt: Point) {
        let (dx, dy) = self.last_mouse_pos.calculate_offset(pt);

        if let Some(win) = self.selected {
            self.desktop.current_mut().set_floating(
                &self.conn,
                win,
                self.screens.focused().unwrap(),
            );
        }

        let current = self.desktop.current_mut();

        if let Some(win) = self.selected {
            if let Some(win) = current.windows.lookup_mut(win) {
                win.do_resize(&self.conn, dx, dy);
            } else {
                error!("Tried to move untracked window {}", win)
            }
        } else {
            warn!("Nothing selected");
        }

        self.last_mouse_pos = pt;
    }

    /// Warps the window in the direction passed to it.
    pub fn warp_window(&mut self, dist: i32, dir: Cardinal) {
        //todo: this still affects master window
        if let Some(id) = self.focused_client_id() {
            self.desktop.current_mut().set_floating(
                &self.conn,
                id,
                self.screens.focused().unwrap(),
            );
        }

        let current = self.desktop.current_mut();

        if let Some(win) = current.focused_client_mut() {
            match dir {
                Cardinal::Up => win.do_move(&self.conn, 0, -dist),
                Cardinal::Down => win.do_move(&self.conn, 0, dist),
                Cardinal::Left => win.do_move(&self.conn, -dist, 0),
                Cardinal::Right => win.do_move(&self.conn, dist, 0),
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

    //* Private methods
    fn process_next_event(&mut self) -> Result<Option<Vec<EventAction>>> {
        let Some(event) = self.conn.poll_next_event()? else {return Ok(None)};
        let actions = EventAction::from_xevent(event, self.state());
        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    #[instrument(level = "debug", skip(self, actions, mousebinds, keybinds))]
    fn handle_event(
        &mut self,
        actions: Vec<EventAction>,
        mousebinds: &mut Mousebinds<X>,
        keybinds: &mut Keybinds<X>,
    ) -> Result<()> {
        use EventAction::*;

        for action in actions {
            match action {
                ClientFocus(id) => self.update_focus(id)?,
                ClientUnfocus(id) => self.client_unfocus(id)?,
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
                RunMousebind(mb, id, pt) => self.run_mousebind(mb, mousebinds, id, pt),
                ToggleClientFullscreen(id, should_fs) => self.set_fullscreen(id, should_fs)?,
                ToggleUrgency(id) => self.toggle_urgency(id)?,
                HandleError(err, evt) => self.handle_error(err, evt),
            }
        }

        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
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
        self.focused = Some(target);
        self.desktop.current_mut().focus_window(&self.conn, target);
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn focused_client_id(&self) -> Option<XWindowID> {
        self.desktop.current_client().map(|c| c.id())
    }

    /// Unfocuses a client
    #[instrument(level = "debug", skip(self))]
    fn client_unfocus(&mut self, id: XWindowID) -> Result<()> {
        self.desktop.current_mut().unfocus_window(&self.conn, id);
        self.focused = self.desktop.current_client().map(|c| c.id());
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
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
    #[instrument(level = "debug", skip(self))]
    fn client_name_change(&mut self, id: XWindowID) -> Result<()> {
        //todo
        if let Some(c) = self.desktop.current_client_mut() {
            c.update_dynamic(&self.conn);
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn map_tracked_client(&mut self, id: XWindowID) -> Result<()> {
        let current = self.desktop.current_mut();
        if self.conn.should_float(id, &self.config.float_classes) || current.is_floating() {
            current.add_window_floating(&self.conn, self.screens.focused().unwrap(), id)
        } else {
            current.add_window_tiled(&self.conn, self.screens.focused().unwrap(), id)
        }
        Ok(())
    }

    fn map_untracked_client(&self, id: XWindowID) -> Result<()> {
        Ok(self.conn.map_window(id)?)
    }

    #[instrument(level = "debug", skip(self))]
    fn unmap_client(&mut self, id: XWindowID) -> Result<()> {
        self.desktop
            .current_mut()
            .del_window(&self.conn, self.screens.focused().unwrap(), id)?;
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn configure_client(&mut self, data: ConfigureRequestData) -> Result<()> {
        Ok(())
    }

    #[instrument(level = "debug", skip(self))]
    fn client_to_workspace(&mut self, id: XWindowID, idx: usize) -> Result<()> {
        let name = match self.desktop.get(idx) {
            Some(ws) => ws.name.to_string(),
            None => return Ok(()),
        };

        self.desktop
            .send_window_to(id, &name, &self.conn, self.screens.focused().unwrap())
    }

    #[instrument(level = "debug", skip(self, bdgs))]
    fn run_keybind(&mut self, kb: Keybind, bdgs: &mut Keybinds<X>, id: XWindowID) {
        if let Some(focused) = self.focused {
            if focused != id {
                warn!("Keypress event and focused window are different");
            }
        }
        if let Some(cb) = bdgs.get_mut(&kb) {
            cb(self);
        } else {
            warn!("Binding not found for keypress event");
        }
    }

    #[instrument(level = "debug", skip(self, bdgs))]
    fn run_mousebind(&mut self, mb: Mousebind, bdgs: &mut Mousebinds<X>, id: XWindowID, pt: Point) {
        match mb.kind {
            MouseEventKind::Press => {
                self.conn
                    .grab_pointer(self.root.id, 0)
                    .unwrap_or_else(|e| error!("{}", e));
                self.selected = Some(id);
                self.last_mouse_pos = pt;
            }
            MouseEventKind::Release => {
                self.conn
                    .ungrab_pointer()
                    .unwrap_or_else(|e| error!("{}", e));
                self.selected = None;
                self.last_mouse_pos = pt;
            }
            MouseEventKind::Motion => {}
        }

        if let Some(focused) = self.focused {
            if focused != id {
                warn!("Mouse event and focused window are different");
            }
        }
        if let Some(cb) = bdgs.get_mut(&mb) {
            cb(self, pt);
        } else {
            warn!("Binding not found for mouse event");
        }
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
