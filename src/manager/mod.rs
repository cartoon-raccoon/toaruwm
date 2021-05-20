use crate::x::{
    XConn, 
    XWindowID,
};
use crate::types::MouseMode;
use crate::core::{Screen, Desktop};

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

}