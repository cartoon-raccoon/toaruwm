//! Input event handling functionality for Wayland.

use smithay::backend::input::{InputBackend, InputEvent};
use smithay::utils::{SERIAL_COUNTER, Point, Logical};

use crate::platform::wayland::prelude::*;

use super::PointerFocusTarget;

#[doc(hidden)]
impl<C: RuntimeConfig, B: WaylandBackend> Wayland<C, B> {
    pub(crate) fn handle_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
            InputEvent::DeviceAdded { device } => {}
            InputEvent::Keyboard { event } => self.on_keyboard_event::<I>(event),
            InputEvent::PointerMotion { event } => self.on_pointer_motion::<I>(event),
            InputEvent::PointerMotionAbsolute { event } => self.on_pointer_motion_absolute::<I>(event),
            InputEvent::PointerButton { event } => self.on_pointer_button::<I>(event),
            InputEvent::PointerAxis { event } => self.on_pointer_axis::<I>(event),
            _ => {
            }
        }
    }

    pub(crate) fn on_keyboard_event<I: InputBackend>(&mut self, event: I::KeyboardKeyEvent) {

    }

    pub(crate) fn on_pointer_motion<I: InputBackend>(&mut self, event: I::PointerMotionEvent) {
        let mut pointer_loc = self.wl.pointer.current_location();

        let serial = SERIAL_COUNTER.next_serial();


    }

    pub(crate) fn on_pointer_motion_absolute<I: InputBackend>(&mut self, event: I::PointerMotionAbsoluteEvent) {

    }

    pub(crate) fn on_pointer_button<I: InputBackend>(&mut self, event: I::PointerButtonEvent) {

    }

    pub(crate) fn on_pointer_axis<I: InputBackend>(&mut self, event: I::PointerAxisEvent) {

    }

    pub(crate) fn surface_under_pointer(
        &self, 
        pos: Point<f64, Logical>,
    ) -> Option<(PointerFocusTarget, Point<f64, Logical>)> {
        let output = self.wl.global_space.outputs().find(|o| {
            let geom = self.wl.global_space.output_geometry(o).unwrap();
            geom.contains(pos.to_i32_round())
        })?;

        let output_geom = self.wl.global_space.output_geometry(output).unwrap();
        
        todo!()
    }
}