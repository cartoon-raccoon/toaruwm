use super::{XWindowID, XConn};
use crate::core::types::{Atom, NetWindowStates};

pub trait Ewmh: XConn {
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<Atom>>;
    fn get_window_states(&self, window: XWindowID) -> NetWindowStates;
    fn set_supported(&self, screen_idx: i32, atoms: &[Atom]);
    fn set_wm_state(&self, window: XWindowID, atoms: &[Atom]);
}