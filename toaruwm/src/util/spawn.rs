//! Utilities for spawning external commands.

use std::thread;
use std::process::Command;

use tracing::instrument;

/// Spawns a new command in a separate thread.
pub fn spawn<S, I>(cmd: Command) -> Result<(), std::io::Error> {
    let _res = thread::Builder::new()
        .name("Command Spawner".to_owned())
        .spawn(move || {
            spawn_in_thread(cmd);
        })?;

    Ok(())
}

#[instrument(level = "trace", skip_all)]
fn spawn_in_thread(cmd: Command) {

}