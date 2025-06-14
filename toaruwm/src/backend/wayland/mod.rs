#![allow(missing_docs)] // fixme: remove when we get a working prototype

use thiserror::Error;

use smithay::reexports::{
    wayland_server::{
        Display,
        backend::ClientId as WlClientId,
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
use self::backend::{
    WaylandBackend,
    drm::{DrmBackend, DrmError},
};
use crate::core::types::ToaruClientId;

/// An ID representing a Wayland client.
/// 
/// Implements [`ToaruClientId`].
pub type ClientId = WlClientId;

impl ToaruClientId for ClientId {}

/// The 
#[derive(Debug)]
pub struct Wayland<B: WaylandBackend> {
    pub(crate) display: Display<WlState>,
    pub(crate) backend: B,
}

impl<B: WaylandBackend> Wayland<B> {
    /// Creates a new Wayland compositor, autoselecting the appropriate
    /// backend based on the current state of the system.
    pub fn new() -> Result<Wayland<B>, WaylandError> {
        let backend = backend::backend_autocreate()?;

        todo!()
    }

    /// Creates a new Wayland compositor, backed by a [`DrmBackend`].
    pub fn new_with_drm() -> Result<Wayland<DrmBackend>, WaylandError> {
        let backend = DrmBackend::new()?;

        todo!()
    }

    pub fn new_with_winit() -> Result<Self, WaylandError> {
        todo!()
    }
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