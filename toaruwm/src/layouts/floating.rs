use super::{update::Update, Layout, LayoutAction, LayoutCtxt, LayoutType};

use crate::{platform::Platform, types::ClientId};

/// A simple floating layout that does not
/// enforce any specific window layout.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Floating {}

impl Floating {
    /// Creates a new floating layout.
    pub fn new() -> Self {
        Self {}
    }
}

impl<P: Platform> Layout<P> for Floating {
    fn name(&self) -> &str {
        "Floating"
    }

    fn boxed(&self) -> Box<dyn Layout<P>> {
        Box::new(*self)
    }

    fn layout(&self, _: LayoutCtxt<'_, P>) -> Vec<LayoutAction<'_, P>> {
        vec![]
    }

    fn receive_update(&self, _: &Update) {
        /* nothing but the vast emptiness of the void :') */
    }

    fn style(&self) -> LayoutType {
        LayoutType::Floating
    }
}
