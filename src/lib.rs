#[macro_use]
mod log;

pub mod x;
pub mod core;
pub mod layouts;
pub mod manager;
pub mod keybinds;

pub(crate) mod util;

pub use crate::core::types;
pub use crate::manager::WindowManager;

