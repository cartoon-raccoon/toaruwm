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
use toaruwm::ToaruManagerConfig;
use toaruwm::platform::wayland::Wayland;
use toaruwm::Toaru;
// use flexi_logger::{
//     Logger,
//     LogSpecification,
// };
use tracing::Level;
use tracing_subscriber::{fmt as logger, fmt::format::FmtSpan};

fn main() -> Result<(), Box<dyn Error>> {
    // init logger
    logger::fmt()
        .with_span_events(FmtSpan::ACTIVE)
        .with_max_level(Level::TRACE)
        .try_init().expect("could not create logger");

    // create config
    let config = ToaruManagerConfig::builder().finish(|_| Ok(()))?;

    // create the event loop
    let event_loop = EventLoop::try_new()?;

    // create the backend for the Wayland config
    let backend = DrmBackend::new(event_loop.handle())?;
    
    // create the overall Toaru struct
    let toaru: Toaru<Wayland<_, DrmBackend<_>>, _> = Toaru::new(config)?;

    // create the platform driven by the backend we made earlier
    let (mut platform, display) = Wayland::new(
        backend,
        None,
        toaru, 
        event_loop.handle(), 
        event_loop.get_signal()
    )?;

    platform.run(display, event_loop)?;


    Ok(())
}