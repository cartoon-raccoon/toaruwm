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

/// Update the internally stored border thickness.
/// 
/// Some tiling layouts may have to account for the user-set
/// border thickness when calculating window geometries.
/// This update tells them to change their internal store
/// of border thickness so they can update their calculations
/// accordingly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UpdateBorderPx(pub u32);
impl IntoUpdate for UpdateBorderPx {}

/// Cycle the shown window of the layout in the given direction.
/// 
/// Some layouts (like a monocle layout) might keep only one window
/// mapped at time, taking up the entire screen, while the rest
/// of the windows are unmapped.
/// 
/// Sending this message to the layout tells it to cycle to the next
/// window to focus to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CycleFocused(pub Direction);
impl IntoUpdate for CycleFocused {}
