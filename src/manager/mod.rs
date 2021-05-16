use crate::x::{
    XConn, 
    XWindowID,
    Ewmh, Icccm,
};
use crate::types::MouseMode;
use crate::core::desktop::Screen;

/// The main window manager object that receives and responds to events.
#[allow(dead_code)]
pub struct WindowManager<X>
where X: XConn + Ewmh + Icccm {
    pub(crate) conn: X,
    //todo: add these
    //pub(crate) desktop: Desktop,
    pub(crate) screen: Screen,
    root: i32,
    mousemode: MouseMode,
    selected: Option<XWindowID>,
    last_mouse_x: i32,
    last_mouse_y: i32,
    to_quit: bool,
}

impl<X: XConn + Ewmh + Icccm> WindowManager<X> {

}