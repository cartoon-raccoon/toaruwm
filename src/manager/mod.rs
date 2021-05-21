use crate::x::{
    XConn, 
    XWindowID,
};
use crate::types::MouseMode;
//use crate::layouts::LayoutType;
use crate::core::{Screen, Desktop};
use crate::util;

/// The main window manager object that receives and responds to events.
#[allow(dead_code)]
pub struct WindowManager<X: XConn> {
    pub(crate) conn: X,
    pub(crate) desktop: Desktop,
    pub(crate) screen: Screen,
    root: i32,
    mousemode: MouseMode,
    selected: Option<XWindowID>,
    last_mouse_x: i32,
    last_mouse_y: i32,
    to_quit: bool,
}

impl<X: XConn> WindowManager<X> {

    // pub fn new(conn: X) -> WindowManager<X> {
    //     let root_id = conn.get_root();
    //     Self {
    //         conn: conn,
    //         desktop: Desktop::new(LayoutType::Floating),
    //         screen: Screen::new(root_id),
    //     }
    // }

    pub fn register(conn: X) -> Self {
        let root_id = conn.get_root();

        debug!("Got root id of {}", root_id);

        conn.change_window_attributes(root_id, &util::ROOT_ATTRS)
        .unwrap_or_else(|_| {
            error!("Another window manager is running.");
            std::process::exit(1)
        });

        //conn.set_supported(sc);

        todo!()
    }
}