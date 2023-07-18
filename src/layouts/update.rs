use std::any::Any;

use crate::types::Direction;

/// A dynamically typed object that can change the behavior of a layout.
pub struct Update(Box<dyn Any>);

use std::fmt;
impl fmt::Debug for Update {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Update <type erased>")
    }
}

impl Update {
    /// Creates a new Update.
    pub fn new<U: 'static>(u: U) -> Self {
        Self(Box::new(u))
    }

    /// Tries to downcast self as a Box of the associated object.
    pub fn as_update<U: 'static>(&self) -> Option<&U> {
        self.0.downcast_ref()
    }
}

/// Marker trait to indicate that a type can be sent
/// as an update to a Layout.
pub trait IntoUpdate: Any
where
    Self: Sized,
{
    /// Converts Self into an update.
    fn into_update(self) -> Update {
        Update::new(self)
    }
}

/// Resize the main window of the layout by the given increment/decrement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeMain(pub f32);
impl IntoUpdate for ResizeMain {}

/// Cycle the shown window of the layout in the given direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CycleFocused(pub Direction);
impl IntoUpdate for CycleFocused {}
