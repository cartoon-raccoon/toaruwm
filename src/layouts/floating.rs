use super::LayoutAction;

use crate::core::{Workspace, Screen};

// Ws and scr are only included to satisfy the function signature.
pub(super) fn gen_layout(_: &Workspace, _: &Screen, _: u32, _: f32) -> Vec<LayoutAction> {
    Vec::new()
}