use toaruwm::xcb_backed_wm;

pub fn main() {
    let mut wm = xcb_backed_wm();

    wm.register();
    wm.run();
}