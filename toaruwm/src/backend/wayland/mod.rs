#![allow(missing_docs)]

use smithay::reexports::{
    wayland_server::{
        Display,
    },
};

pub mod state;
pub mod handlers;
pub mod backend;

pub use state::WlState;

#[derive(Debug)]
pub struct Wayland {
    pub(crate) display: Display<WlState>,

}