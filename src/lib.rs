#[macro_use]
extern crate bitflags;

#[macro_use]
mod log;

pub mod x;
pub mod core;
pub mod layouts;
pub mod manager;
pub mod keybinds;

pub(crate) mod util;

pub use crate::core::types;
pub use crate::core::types::Result;
pub use crate::x::core::Result as XResult;
pub use crate::manager::WindowManager;

use crate::x::xcb::XCBConn;

/// Convenience function for creating a XCB-backed WindowManager.
pub fn xcb_backed_wm() -> XResult<WindowManager<XCBConn>> {
    let mut xcbconn = XCBConn::connect()?;
    xcbconn.init()?;

    let wm = WindowManager::new(xcbconn);

    Ok(wm)
}

