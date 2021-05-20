use super::ResizeAction;

use crate::core::{Workspace, Screen};

// Ws and scr are only included to satisfy the function signature.
pub(super) fn gen_layout(_ws: &Workspace, _scr: &Screen) -> Vec<ResizeAction> {
    Vec::new()
}