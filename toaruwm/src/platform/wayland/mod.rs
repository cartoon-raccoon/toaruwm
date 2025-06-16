#![allow(missing_docs)] // fixme: remove when we get a working prototype

use thiserror::Error;

use smithay::reexports::{
    calloop::{
        EventLoop,
        Error as CalloopError,
    }, 
    wayland_server::{
        backend::ClientId as WlsClientId,
        Display
    }
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

use self::backend::{
    WaylandBackend,
    drm::{DrmBackend, DrmError},
};

use crate::core::types::ClientId;

/// An identifier corresponding to a Wayland client.
pub type WaylandClientId = WlsClientId;

impl ClientId for WaylandClientId {}

/// An implementation of the Wayland platform.
#[derive(Debug)]
pub struct Wayland<B: WaylandBackend> {
    pub(crate) display: Display<WlState>,
    pub(crate) state: WlState,
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
    #[error(transparent)]
    EventLoopErr(CalloopError),
}

impl From<CalloopError> for WaylandError {
    // fixme: 
    fn from(e: CalloopError) -> WaylandError {
        WaylandError::EventLoopErr(e)
    }
}