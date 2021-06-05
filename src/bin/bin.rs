use std::error::Error;

use toaruwm::xcb_backed_wm;

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut wm = xcb_backed_wm()?;

    wm.register(Vec::new());
    wm.run()?;

    Ok(())
}