#![allow(missing_docs)]

use thiserror::Error;

use smithay::reexports::{
    wayland_server::{
        Display,
    },
};

use smithay::backend::{
    session::{
        libseat::Error as SeatError,
    }
};

pub mod state;
pub mod handlers;
pub mod backend;

pub use state::WlState;

use super::BackendError;

use backend::drm::DrmError;

#[derive(Debug)]
pub struct Wayland {
    pub(crate) display: Display<WlState>,

}

#[derive(Debug, Error)]
pub enum WaylandError {
    #[error("unable to establish seat: {0}")]
    SessionErr(SeatError),
    #[error("udev failure: {0}")]
    UdevErr(String),
    #[error(transparent)]
    DrmError(DrmError),
}

impl From<WaylandError> for BackendError {
    fn from(e: WaylandError) -> BackendError {
        BackendError::WaylandError(e)
    }
}