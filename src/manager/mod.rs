//#![allow(unused_variables, unused_imports, dead_code)]
use std::process::{Command, Stdio};
use std::iter::FromIterator;
use std::fmt;

use crate::{
    Result, ToaruError, ErrorHandler,
};
use crate::log::basic_error_handler;
use crate::x::{
    XConn, 
    XError,
    XEvent,
    XWindow,
    XWindowID,
    Atom,
};
use crate::types::{
    Ring,
    MouseMode, Direction,
    ClientAttrs,
};
use crate::keybinds::{
    Mousebinds,
    Mousebind,
    Keybinds, 
    Keybind,
};
use crate::layouts::LayoutType;
use crate::core::{Screen, Desktop};

pub mod event;
pub mod state;
pub mod config;

pub(crate) use state::WMState;
pub use event::EventAction;

use config::Config;

/// Some arbitrary code that can run on a certain event.
/// 
/// Accepts a `&mut WindowManager<X>` as a parameter, so it can
/// manipulate internal manager state.
pub type Hook<X> = Box<dyn FnMut(&mut WindowManager<X>)>;

/// The main window manager object that owns the event loop,
/// and receives and responds to events.
/// 
/// The manager is generic over a type argument X that 
/// implements the `XConn` trait, but is never directly exposed
/// by the type's public API and is only used when constructing
/// a new window manager instance.
pub struct WindowManager<X: XConn> {
    /// The X Connection
    conn: X,
    config: Config,
    /// The desktop containing all windows.
    desktop: Desktop,
    screens: Ring<Screen>,
    root: XWindow,
    ehandler: ErrorHandler,
    mousemode: MouseMode,
    /// The window currently being manipulated
    /// if `self.mousemode` is not None.
    selected: Option<XWindowID>,
    /// The window currently in focus.
    focused: Option<XWindowID>,
    last_workspace: usize,
    last_mouse_x: i32,
    last_mouse_y: i32,
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
            .field("focused", &self.focused)
            .finish()
    }
}

impl<X: XConn> WindowManager<X> {

    /// Constructs a new WindowManager object.
    pub fn new(conn: X) -> WindowManager<X> {
        fn_ends!("WindowManager::new");

        let root = conn.get_root();
        let mut screens = Ring::from_iter(conn.all_outputs().unwrap_or_else(
            |e| fatal!("Could not get screens: {}", e)
        ));
        let config = Config::default();
        let workspaces = config.workspaces.clone();

        debug!("Got screens: {:?}", screens);
        screens.set_focused(0);

        Self {
            conn,
            config,
            desktop: Desktop::new(
                LayoutType::Floating, None, 
                workspaces
            ),
            screens,
            root,
            ehandler: Box::new(basic_error_handler),
            mousemode: MouseMode::None,
            selected: None,
            focused: None,
            last_workspace: 0,
            last_mouse_x: 0,
            last_mouse_y: 0,
            running: true,
            restart: false,
        }
    }

    //* Public Methods

    /// Registers the executable as a window manager
    /// with the X server.
    /// 
    /// Selects for subtructure redirect and notify,
    /// grabs required keys for keybinds,
    /// and runs any registered startup hooks.
    pub fn register(&mut self, hooks: Vec<Hook<X>>) {
        fn_ends!("WindowManager::init");

        let root = self.conn.get_root();

        debug!("Got root window data: {:?}", root);

        self.conn.change_window_attributes(root.id, &[ClientAttrs::RootEventMask])
        .unwrap_or_else(|_| {
            error!("Another window manager is running.");
            std::process::exit(1)
        });

        self.conn.set_supported(&[
            Atom::WmProtocols,
            Atom::WmTakeFocus,
            Atom::WmState,
            Atom::WmDeleteWindow,
        ]).unwrap_or_else(|e| {
            error!("{}", e);
            std::process::exit(1)
        });

        // run hooks
        for mut hook in hooks {
            hook(self);
        }
    }

    pub fn grab_and_run(&mut self, 
        mb: Mousebinds<X>, kb: Keybinds<X>
    ) -> Result<()> {
        self.grab_bindings(&mb, &kb)?;
        self.run(mb, kb)
    }

    pub fn grab_bindings(&mut self, 
        mb: &Mousebinds<X>, 
        kb: &Keybinds<X>
    ) -> Result<()> {
        let root_id = self.conn.get_root().id;
        for (binding, _) in mb {
            self.conn.grab_button(*binding, root_id, true)?;
        }
        
        for (binding, _) in kb {
            self.conn.grab_key(*binding, root_id)?;
        }

        Ok(())
    }

    /// Runs the main event loop.
    pub fn run(
        &mut self,
        mut mb: Mousebinds<X>,
        mut kb: Keybinds<X>
    ) -> Result<()> {
        fn_ends!("WindowManager::run");

        loop {
            let event = self.process_next_event().or_else(|e|{
                match e {
                    // only return if the error is a connection error
                    ToaruError::XConnError(XError::Connection(_)) => Err(e),
                    // else, handle error and map to an Ok(None)
                    e => {(self.ehandler)(e); Ok(None)}
                }
            })?;
            if let Some(actions) = event {
                // if event handling returned an error, do not return
                // instead, handle it internally and continue
                if let Err(e) = self.handle_event(actions, &mut mb, &mut kb) {
                    (self.ehandler)(e);
                }
            }

            //* update window properties

            if !self.running {break}
        }

        if self.restart {
            todo!("restart process")
        }

        Ok(())
    }

    /// Run an external command.
    pub fn run_external<S: Into<String>>(&mut self, cmd: S, args: &[&str]) {
        let cmd = cmd.into();
        debug!("Running command [{}] with args {:?}", cmd, args);
        let result = Command::new(cmd)
            .args(args)
            .stdout(Stdio::null())
            .stdout(Stdio::null())
            .spawn();

        match result {
            Ok(_) => {},
            Err(e) => {
                (self.ehandler)(ToaruError::SpawnProc(e.to_string()))
            }
        }
    }

    /// Set an error handler for WindowManager.
    pub fn set_error_handler<F>(&mut self, f: F) 
    where 
        F: FnMut(ToaruError) + 'static {
        self.ehandler = Box::new(f);
    }
    
    /// Goes to the specified workspace.
    pub fn goto_workspace(&mut self, name: &str) {
        self.desktop.goto(name, &self.conn, self.screens.focused().unwrap());
    }
    
    /// Cycles the focused workspace.
    pub fn cycle_workspace(&mut self, direction: Direction) {
        self.desktop.cycle_workspace(&self.conn, self.screens.focused().unwrap(), direction);
    }

    /// Sends a window to the specified workspace.
    pub fn send_window_to(&mut self, name: &str) {
        self.desktop.send_window_to(name, &self.conn, self.screens.focused().unwrap());
    }
    
    /// Sends a window to the specified workspace and then switches to it.
    pub fn send_window_and_switch(&mut self, name: &str) {
        self.desktop.send_window_to(name, &self.conn, self.screens.focused().unwrap());
        self.desktop.goto(name, &self.conn, self.screens.focused().unwrap());
    }

    /// Cycles the focused window.
    pub fn cycle_focus(&mut self, direction: Direction) {
        self.desktop.current_mut().cycle_focus(&self.conn, direction);
    }

    /// Toggles the state of the focused window to floating or vice versa.
    pub fn toggle_focused_state(&mut self) {
        self.desktop.current_mut().toggle_focused_state(&self.conn, self.screens.focused().unwrap());
    }

    /// Grabs the pointer and moves the window the pointer is on.
    pub fn move_window_ptr(&mut self) {
        todo!()
    }

    /// Grabs the pointer and resizes the window the pointer is on.
    /// 
    /// If the window is tiled, its state is toggled to floating
    /// and the entire desktop is re-laid out.
    pub fn resize_window_ptr(&mut self) {
        todo!()
    }

    /// Closes the focused window
    pub fn close_focused_window(&mut self) {
        if let Some(window) = self.desktop.current_mut().windows.focused() {
            if let Err(e) = self.conn.destroy_window(window.id()) {
                (self.ehandler)(e.into())
            }
        }
    }

    /// Quits the event loop.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn restart(&mut self) {
        self.running = false;
        self.restart = true;
    }

    pub fn dump_internal_state(&self) {
        info!("========== | Internal State Dump | ==========");
        info!("{:#?}", &self);
    }

    //* Private methods
    fn process_next_event(&mut self) -> Result<Option<Vec<EventAction>>> {
        if let Some(event) = self.conn.poll_next_event()? {
            Ok(Some(EventAction::from_xevent(event, self.state())))
        } else {
            Ok(None)
        }
    }

    fn handle_event(
        &mut self, 
        actions: Vec<EventAction>,
        mousebinds: &mut Mousebinds<X>,
        keybinds: &mut Keybinds<X>,
    ) -> Result<()> {
        use EventAction::*;

        for action in actions {
            match action {
                ClientFocus(id) => {self.update_focus(id)?}
                ClientUnfocus(id) => {self.client_unfocus(id)?},
                ClientNameChange(id) => {self.client_name_change(id)?},
                ScreenReconfigure => {self.screen_reconfigure()?},
                SetFocusedScreen(pt) => {},
                DestroyClient(id) => {},
                MapTrackedClient(id) => {self.map_tracked_client(id)?},
                MapUntrackedClient(id) => {self.map_untracked_client(id)?},
                UnmapClient(id) => {self.unmap_client(id)?},
                ConfigureClient(id, geom) => {},
                RunKeybind(kb, id) => {self.run_keybind(kb, keybinds, id)},
                RunMousebind(mb, id, _) => {self.run_mousebind(mb, mousebinds, id)},
                ToggleClientFullscreen(id, thing) => {},
                ToggleUrgency(id) => {},
                HandleError(err, evt) => {self.handle_error(err, evt)},
            }
        }

        Ok(())
    }

    fn update_focus(&mut self, id: XWindowID) -> Result<()> {
        fn_ends!("update_focus for window {}", id);
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
                None => /*handle this*/ return Err(ToaruError::UnknownClient(id))
            }
        };
        self.focused = Some(target);
        self.desktop.current_mut().focus_window(&self.conn, target);
        Ok(())
    }

    fn focused_client_id(&self) -> Option<XWindowID> {
        self.desktop.current_client().map(|c| c.id())
    }

    /// Unfocuses a client
    fn client_unfocus(&mut self, id: XWindowID) -> Result<()> {
        fn_ends!("lost focus for window {}", id);

        self.desktop.current_mut().unfocus_window(&self.conn, id);
        self.focused = self.desktop.current_client().map(|c| c.id());
        Ok(())
    }

    /// Query _NET_WM_NAME or WM_NAME and change it accordingly
    fn client_name_change(&mut self, id: XWindowID) -> Result<()> {
        todo!()
    }

    fn map_tracked_client(&mut self, id: XWindowID) -> Result<()> {
        fn_ends!("Wm::map_tracked_client({})", id);

        let current = self.desktop.current_mut();
        if self.conn.should_float(id, &self.config.float_classes) ||
            current.is_floating() {
            current.add_window_floating(
                &self.conn, self.screens.focused().unwrap(), id
            )
        } else {
            current.add_window_tiled(
                &self.conn, self.screens.focused().unwrap(), id
            )
        }
        Ok(())
    }

    fn map_untracked_client(&self, id: XWindowID) -> Result<()> {
        Ok(self.conn.map_window(id)?)
    }

    fn unmap_client(&mut self, id: XWindowID) -> Result<()> {
        fn_ends!("Wm::unmap_tracked_client({})", id);

        self.desktop.current_mut().del_window(
            &self.conn,
            &self.screens.focused().unwrap(),
            id,
        )?;
        Ok(())
    }

    fn run_keybind(&mut self, 
        kb: Keybind, bdgs: &mut Keybinds<X>, id: XWindowID
    ) {
        if let Some(focused) = self.focused {
            if focused != id {
                warn!("Keypress event and focused window are different")
            }
        }
        if let Some(cb) = bdgs.get_mut(&kb) {
            cb(self);
        }
    }

    fn run_mousebind(&mut self, 
        mb: Mousebind, bdgs: &mut Mousebinds<X>, id: XWindowID
    ) {
        if let Some(focused) = self.focused {
            if focused != id {
                warn!("Mouse event and focused window are different")
            }
        }
        if let Some(cb) = bdgs.get_mut(&mb) {
            cb(self);
        }
    }

    fn screen_reconfigure(&mut self) -> Result<()> {
        todo!()
    }

    fn handle_error(&mut self, err: XError, _evt: XEvent) {
        (self.ehandler)(ToaruError::XConnError(err));
    }
}