use std::error::Error;

use toaruwm::xcb_backed_wm;
use toaruwm::keybinds::{
    new_keybinds,
    new_mousebinds,
};

pub fn main() -> Result<(), Box<dyn Error>> {
    //* 1: Setup X Connection and allocate new WM object
    let mut wm = xcb_backed_wm()?;

    //* 2: Read/setup config
    // if using as a library, declare config here
    // else use a Config type to read a config file

    //* 3: Register the WM as a client with the X server
    //*    and initialise internal state
    //* a: Grab keys and mousebinds
    wm.register(Vec::new() /* pass config in here */);

    //* 4: Run the WM
    wm.run(new_mousebinds(), new_keybinds())?;

    Ok(())
}