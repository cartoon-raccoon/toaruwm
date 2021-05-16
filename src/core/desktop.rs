use crate::x::{XWindow, XWindowID};

/// Represents a physical monitor.
#[derive(Clone, Copy)]
pub struct Screen {
    pub xwindow: XWindow,
    pub idx: i32,
}

impl Screen {
    pub fn new(screen_idx: i32, root_id: XWindowID) -> Self {
        Self {
            xwindow: XWindow::from(root_id),
            idx: screen_idx,
        }
    }
}