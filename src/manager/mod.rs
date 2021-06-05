use std::collections::HashMap;
use std::process::Command;

use crate::x::{
    XConn, 
    XWindowID,
};
use crate::types::{
    Result,
    MouseMode, Direction,
};
use crate::keybinds::Keybinds;
use crate::layouts::LayoutType;
use crate::core::{Screen, Desktop};
use crate::util;

pub mod event;
pub mod state;

pub(crate) use state::WMState;
pub use event::EventAction;

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
#[allow(dead_code)]
pub struct WindowManager<X: XConn> {
    pub(crate) conn: X,
    pub(crate) desktop: Desktop,
    pub(crate) screen: Screen,
    pub(crate) root: u32,
    keybinds: Keybinds<X>,
    mousemode: MouseMode,
    selected: Option<XWindowID>,
    last_mouse_x: i32,
    last_mouse_y: i32,
    to_quit: bool,
}

#[allow(dead_code)]
impl<X: XConn> WindowManager<X> {

    /// Constructs a new WindowManager object.
    pub fn new(conn: X) -> WindowManager<X> {
        let root_id = conn.get_root();
        let screens = conn.all_outputs();
        Self {
            conn,
            desktop: Desktop::new(LayoutType::Floating),
            //todo: read up on randr and figure out how the hell this works
            screen: screens[0],
            root: root_id,
            keybinds: HashMap::new(),
            mousemode: MouseMode::None,
            selected: None,
            last_mouse_x: 0,
            last_mouse_y: 0,
            to_quit: false,
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
        let root_id = self.conn.get_root();

        debug!("Got root id of {}", root_id);

        self.conn.change_window_attributes(root_id, &util::ROOT_ATTRS)
        .unwrap_or_else(|_| {
            error!("Another window manager is running.");
            std::process::exit(1)
        });

        //conn.set_supported(sc);

        // run hooks
        for mut hook in hooks {
            hook(self);
        }

        todo!()
    }

    /// Runs the main event loop.
    pub fn run(&mut self) -> Result<()> {
        loop {
            let actions = self.process_next_event();
            self.handle_event(actions)?;

            if self.to_quit {
                break Ok(())
            }
        }
    }

    /// Run an external command.
    pub fn run_external(&mut self, args: &'static [&str]) {
        
        todo!()
    }
    
    /// Goes to the specified workspace.
    pub fn goto_workspace(&mut self, name: &str) {
        self.desktop.goto(name, &self.conn, &self.screen);
    }
    
    /// Cycles the focused workspace.
    pub fn cycle_workspace(&mut self, direction: Direction) {
        self.desktop.cycle_workspace(&self.conn, &self.screen, direction);
    }

    /// Sends a window to the specified workspace.
    pub fn send_window_to(&mut self, name: &str) {
        self.desktop.send_window_to(name, &self.conn, &self.screen);
    }
    
    /// Sends a window to the specified workspace and then switches to it.
    pub fn send_window_and_switch(&mut self, name: &str) {
        self.desktop.send_window_to(name, &self.conn, &self.screen);
        self.desktop.goto(name, &self.conn, &self.screen);
    }

    /// Cycles the focused window.
    pub fn cycle_focus(&mut self, direction: Direction) {
        self.desktop.current_mut().cycle_focus(&self.conn, direction);
    }

    /// Toggles the state of the focused window to floating or vice versa.
    pub fn toggle_focused_state(&mut self) {
        self.desktop.current_mut().toggle_focused_state(&self.conn, &self.screen);
    }

    /// Grabs the pointer and moves the window the pointer is on.
    pub fn move_window_ptr(&mut self) {
        todo!()
    }

    /// Grabs the pointer and resizes the window the pointer is on.
    /// 
    /// If the window is tiled, its state is toggled to floating.
    pub fn resize_window_ptr(&mut self) {
        todo!()
    }

    /// Closes the focused window
    pub fn close_focused_window(&mut self) {
        if let Some(window) = self.desktop.current_mut().windows.focused() {
            self.conn.destroy_window(&window);
        }
    }

    /// Quits the event loop.
    pub fn quit(&mut self) {
        self.to_quit = true;
    }

    //* Private methods
    pub(crate) fn process_next_event(&mut self) -> Vec<EventAction> {
        EventAction::from_xevent(
            self.conn.get_next_event(), 
            self.state()
        )
    }

    fn handle_event(&mut self, actions: Vec<EventAction>) -> Result<()> {
        for _action in actions {
            todo!()  //* match events and run functions accordingly
        }

        Ok(())
    }
}