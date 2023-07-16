use super::{Layout, LayoutAction, LayoutType, update::Update};

use crate::core::{Screen, Workspace};


/// A simple floating layout that does not
/// enforce any specific layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Floating {}

impl Floating {
    /// Creates a new floating layout.
    pub fn new() -> Self {
        Self {}
    }
}

impl Layout for Floating {
    fn name(&self) -> &str {
        "Floating"
    }

    fn boxed(&self) -> Box<dyn Layout> {
        Box::new(*self)
    }

    fn layout(&self, _: &Workspace, _: &Screen) -> Vec<LayoutAction> {
        vec![]
    }

    fn receive_update(&self, _: &Update) {
        ()
    }

    fn style(&self) -> LayoutType {
        LayoutType::Floating
    }
}
