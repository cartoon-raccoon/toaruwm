//! This is placeholder code for the purpose of testing ToaruWM's
//! functionality. A lot of this will be hidden from the user end
//! through procedural macros, so as to provide a more user-friendly
//! interface to configuring ToaruWM via source code.
//!
//! The rest of the comments are a tour through the behind-the-scenes
//! of how ToaruWM is configured.
//!
#![allow(clippy::type_complexity)]

use std::error::Error;

use toaruwm::reexports::{
    calloop::EventLoop,
};

use toaruwm::platform::wayland::backend::DrmBackend;
use toaruwm::ToaruConfig;
use toaruwm::platform::wayland::Wayland;
use toaruwm::Toaru;
// use flexi_logger::{
//     Logger,
//     LogSpecification,
// };
use tracing::Level;
use tracing_subscriber::{fmt as logger, fmt::format::FmtSpan};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    logger::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_max_level(Level::TRACE)
        .try_init()?;

    let config = ToaruConfig::builder().finish(|_| Ok(()))?;

    let event_loop = EventLoop::try_new()?;

    let backend = DrmBackend::new(event_loop.handle())?;
    
    let toaru = Toaru::new(config)?;

    let (mut platform, display) = Wayland::new(
        backend, 
        toaru, 
        event_loop.handle(), 
        event_loop.get_signal()
    )?;

    platform.init(event_loop.handle(), display.handle())?;

    platform.run(display, event_loop)?;


    Ok(())
}