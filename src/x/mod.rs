//! This module provides ToaruWM's main interface to the X server.
//! It exposes a connection trait and provides the basic methods for 
//! retrieving data from and setting data on the X server, that are 
//! called by other modules within ToaruWM.
//! 
//! This module also exposes extension traits that fulfill ICCCM and EWMH
//! functionality, built on top of the base X Connection. Where such
//! functionality is required, these extension traits are used as bounds
//! instead of the base connection trait.
//! 
//! For concrete implementation of the traits exported here, a basic
//! connection object built on top of the XCB library is provided, and
//! can be found inside the `xcb` module.

pub mod core;
pub mod event;

pub use self::core::{XConn, XWindow, XWindowID, xproto};