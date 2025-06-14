#![allow(missing_docs)]

use thiserror::Error;

use smithay::reexports::{
    wayland_server::{
        Display,
    },
};

pub mod state;
pub mod handlers;
pub mod backend;

pub use state::WlState;

use super::BackendError;

#[derive(Debug)]
pub struct Wayland {
    pub(crate) display: Display<WlState>,

}

#[derive(Debug, Error, Clone)]
pub enum WaylandError {
    
}

impl From<WaylandError> for BackendError {
    fn from(e: WaylandError) -> BackendError {
        BackendError::WaylandError(e)
    }
}