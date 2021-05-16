use super::{XWindowID, XConn};
use crate::types::{
    XWinProperties, WindowState, Atom,
    WmHints, SizeHints,
};

/// Exposes ICCCM functionality for an object holding an X connection.
/// For more information on what these methods return, consult the
/// [ICCCM](https://www.x.org/releases/X11R7.6/doc/xorg-docs/specs/ICCCM/icccm.html)
/// reference.
/// 
/// Warning: DO NOT READ THROUGH EVERYTHING. It is incredibly boring and you _will_
/// fall asleep. Consult only the parts you need, as needed.
pub trait Icccm: XConn {
    fn get_client_properties(&self, window: XWindowID) -> XWinProperties;
    fn get_wm_name(&self, window: XWindowID) -> String;
    fn get_wm_icon_name(&self, window: XWindowID) -> String;
    fn get_wm_size_hints(&self, window: XWindowID) -> Option<SizeHints>;
    fn get_wm_hints(&self, window: XWindowID) -> Option<WmHints>;    
    fn get_wm_class(&self, window: XWindowID) -> Option<(String, String)>;
    fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<Atom>>;
    fn get_wm_state(&self, window: XWindowID) -> WindowState;
    fn get_wm_transient_for(&self, window: XWindowID) -> Option<XWindowID>;
    fn get_urgency(&self, window: XWindowID) -> bool;
}