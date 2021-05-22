use crate::x::{
    XConn, 
    XWindowID,
};
use crate::types::{
    MouseMode, Direction,
};
use crate::layouts::LayoutType;
use crate::core::{Screen, Desktop};
use crate::util;

pub mod event;
pub mod state;

pub(crate) use state::WMState;
pub use event::EventAction;

/// The main window manager object that owns the event loop,
/// and receives and responds to events.
#[allow(dead_code)]
pub struct WindowManager<X: XConn> {
    pub(crate) conn: X,
    pub(crate) desktop: Desktop,
    pub(crate) screen: Screen,
    pub(crate) root: u32,
    mousemode: MouseMode,
    selected: Option<XWindowID>,
    last_mouse_x: i32,
    last_mouse_y: i32,
    to_quit: bool,
}

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
    pub fn register(&mut self) {
        let root_id = self.conn.get_root();

        debug!("Got root id of {}", root_id);

        self.conn.change_window_attributes(root_id, &util::ROOT_ATTRS)
        .unwrap_or_else(|_| {
            error!("Another window manager is running.");
            std::process::exit(1)
        });

        //conn.set_supported(sc);

        todo!()
    }

    /// Runs the main event loop.
    pub fn run(&mut self) {
        todo!("WM events not yet implemented")
    }
    
    /// Goes to the specified workspace.
    pub fn goto_workspace(&mut self, name: &str) {
        self.desktop.goto(&self.conn, &self.screen, name);
    }
    
    /// Cycles the focused workspace.
    pub fn cycle_workspace(&mut self, direction: Direction) {
        self.desktop.cycle_workspace(&self.conn, &self.screen, direction);
    }

    /// Sends a window to the specified workspace.
    pub fn send_window_to(&mut self, name: &str) {
        self.desktop.send_window_to(&self.conn, &self.screen, name);
    }
    
    /// Sends a window to the specified workspace and then switches to it.
    pub fn send_window_and_switch(&mut self, name: &str) {
        self.desktop.send_window_to(&self.conn, &self.screen, name);
        self.desktop.goto(&self.conn, &self.screen, name);
    }

    pub fn cycle_focus(&mut self, direction: Direction) {
        self.desktop.current_mut().cycle_focus(&self.conn, direction);
    }

    pub fn toggle_focused_state(&mut self) {
        self.desktop.current_mut().toggle_focused_state(&self.conn, &self.screen);
    }

    pub fn quit(&mut self) {
        self.to_quit = true;
    }

    //* Private methods
    //todo: implement wm events first
}