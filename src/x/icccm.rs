use super::{XWindowID, XConn};
use crate::types::{
    XWinProperties, WindowState, Atom,
    WmHints, SizeHints,
};

// / Exposes ICCCM functionality for an object holding an X connection.
// / For more information on what these methods return, consult the
// / [ICCCM](https://www.x.org/releases/X11R7.6/doc/xorg-docs/specs/ICCCM/icccm.html)
// / reference.
// / 
// / Warning: DO NOT READ THROUGH EVERYTHING. It is incredibly boring and you _will_
// / fall asleep. Consult only the parts you need, as needed.
// pub trait Icccm: XConn {
    
// }