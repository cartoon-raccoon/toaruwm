//! Utilities for spawning external commands.

use std::ffi::OsStr;
use std::thread;

use tracing::instrument;
use smithay::wayland::xdg_activation::XdgActivationToken;

pub fn spawn<S, I>(
    command: S, 
    args: I, 
    token: Option<XdgActivationToken>) -> Result<(), std::io::Error>
where
    S: AsRef<OsStr> + Send + 'static,
    I: IntoIterator<Item = S> + Send + 'static 
{
    let _res = thread::Builder::new()
        .name("Command Spawner".to_owned())
        .spawn(move || {
            spawn_in_thread(command, args, token);
        })?;

    Ok(())
}

#[instrument(level = "trace", skip_all)]
fn spawn_in_thread<S, I>(command: S, args: I, token: Option<XdgActivationToken>)
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = S>
{

}