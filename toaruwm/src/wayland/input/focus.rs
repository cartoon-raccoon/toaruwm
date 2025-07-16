//! Focus targets.

use std::borrow::Cow;

use smithay::{
    reexports::{wayland_server::{
            protocol::wl_surface::WlSurface,
            backend::ObjectId,
        },
    }
};
use smithay::desktop::WindowSurface;
use smithay::backend::input::KeyState;
use smithay::wayland::seat::WaylandFocus;
use smithay::input::{
    Seat, 
    keyboard::{KeyboardTarget, KeysymHandle, ModifiersState}, 
    pointer::{
        PointerTarget, MotionEvent, RelativeMotionEvent, ButtonEvent, AxisFrame, 
        GestureSwipeBeginEvent,GestureSwipeUpdateEvent, GestureSwipeEndEvent,
        GesturePinchBeginEvent, GesturePinchUpdateEvent, GesturePinchEndEvent,
        GestureHoldBeginEvent, GestureHoldEndEvent,
    },
    touch::{TouchTarget, 
        DownEvent as TouchDownEvent, UpEvent as TouchUpEvent, MotionEvent as TouchMotionEvent,
        ShapeEvent, OrientationEvent
    }
};
use smithay::xwayland::X11Surface;
use smithay::desktop::{Window, LayerSurface, PopupKind};
use smithay::utils::{IsAlive, Serial};

use crate::wayland::prelude::*;

/// A target that can receive focus from a keyboard.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyboardFocusTarget {
    /// An application window.
    Window(Window),
    /// A layer surface.
    Layer(LayerSurface),
    /// A popup.
    Popup(PopupKind),
}

impl IsAlive for KeyboardFocusTarget {
    #[inline]
    fn alive(&self) -> bool {
        use KeyboardFocusTarget::*;

        match self {
            Window(w) => w.alive(),
            Layer(l) => l.alive(),
            Popup(p) => p.alive()
        }
    }
}

/// A target than can receive focus from a pointer.
#[derive(Debug, Clone, PartialEq)]
pub enum PointerFocusTarget {
    /// A Wayland surface.
    WlSurface(WlSurface),

    /// An X11 surface.
    #[cfg(feature = "xwayland")]
    X11Surface(X11Surface),
}

impl IsAlive for PointerFocusTarget {
    fn alive(&self) -> bool {
        use PointerFocusTarget::*;

        match self {
            WlSurface(w) => w.alive(),
            X11Surface(x) => x.alive(),
        }
    }
}

impl From<PointerFocusTarget> for WlSurface {
    #[inline]
    fn from(target: PointerFocusTarget) -> Self {
        target.wl_surface().unwrap().into_owned()
    }
}

impl<M: Manager, B: WaylandBackend<M>> KeyboardTarget<Wayland<M, B>> for KeyboardFocusTarget {
    fn enter(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, keys: Vec<KeysymHandle<'_>>, serial: Serial) {
        match self {
            KeyboardFocusTarget::Window(w) => match w.underlying_surface() {
                WindowSurface::Wayland(wl) => KeyboardTarget::enter(wl.wl_surface(), seat, data, keys, serial),
                #[cfg(feature = "xwayland")]
                WindowSurface::X11(x11) => KeyboardTarget::enter(x11, seat, data, keys, serial),
            },
            KeyboardFocusTarget::Layer(l) => KeyboardTarget::enter(l.wl_surface(), seat, data, keys, serial),
            KeyboardFocusTarget::Popup(p) => KeyboardTarget::enter(p.wl_surface(), seat, data, keys, serial)
        }
    }

    fn leave(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, serial: Serial) {
        match self {
            KeyboardFocusTarget::Window(w) => match w.underlying_surface() {
                WindowSurface::Wayland(wl) => KeyboardTarget::leave(wl.wl_surface(), seat, data, serial),
                #[cfg(feature = "xwayland")]
                WindowSurface::X11(x11) => KeyboardTarget::leave(x11, seat, data, serial),
            },
            KeyboardFocusTarget::Layer(l) => KeyboardTarget::leave(l.wl_surface(), seat, data, serial),
            KeyboardFocusTarget::Popup(p) => KeyboardTarget::leave(p.wl_surface(), seat, data, serial)
        }
    }

    fn key(
            &self,
            seat: &Seat<Wayland<M, B>>,
            data: &mut Wayland<M, B>,
            key: KeysymHandle<'_>,
            state: KeyState,
            serial: Serial,
            time: u32,
        ) {
        match self {
            KeyboardFocusTarget::Window(w) => match w.underlying_surface() {
                WindowSurface::Wayland(wl) => KeyboardTarget::key(wl.wl_surface(), seat, data, key, state, serial, time),
                #[cfg(feature = "xwayland")]
                WindowSurface::X11(x11) => KeyboardTarget::key(x11, seat, data, key, state, serial, time),
            },
            KeyboardFocusTarget::Layer(l) => KeyboardTarget::key(l.wl_surface(), seat, data, key, state, serial, time),
            KeyboardFocusTarget::Popup(p) => KeyboardTarget::key(p.wl_surface(), seat, data, key, state, serial, time)
        }
    }
    
    fn modifiers(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, modifiers: ModifiersState, serial: Serial) {
        match self {
            KeyboardFocusTarget::Window(w) => match w.underlying_surface() {
                WindowSurface::Wayland(wl) => KeyboardTarget::modifiers(wl.wl_surface(), seat, data, modifiers, serial),
                #[cfg(feature = "xwayland")]
                WindowSurface::X11(x11) => KeyboardTarget::modifiers(x11, seat, data, modifiers, serial),
            },
            KeyboardFocusTarget::Layer(l) => KeyboardTarget::modifiers(l.wl_surface(), seat, data, modifiers, serial),
            KeyboardFocusTarget::Popup(p) => KeyboardTarget::modifiers(p.wl_surface(), seat, data, modifiers, serial)
        }
    }
}

impl<M: Manager, B: WaylandBackend<M>> PointerTarget<Wayland<M, B>> for PointerFocusTarget {
    fn enter(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &MotionEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::enter(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::enter(x11, seat, data, event),
        }
    }

    fn motion(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &MotionEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::motion(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::motion(x11, seat, data, event),
        }
    }

    fn relative_motion(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &RelativeMotionEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::relative_motion(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::relative_motion(x11, seat, data, event),
        }
    }

    fn button(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &ButtonEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::button(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::button(x11, seat, data, event),
        }
    }

    fn axis(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, frame: AxisFrame) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::axis(wl, seat, data, frame),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::axis(x11, seat, data, frame),
        }
    }

    fn frame(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::frame(wl, seat, data),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::frame(x11, seat, data),
        }
    }

    fn gesture_swipe_begin(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &GestureSwipeBeginEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::gesture_swipe_begin(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::gesture_swipe_begin(x11, seat, data, event),
        }
    }

    fn gesture_swipe_update(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &GestureSwipeUpdateEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::gesture_swipe_update(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::gesture_swipe_update(x11, seat, data, event),
        }
    }

    fn gesture_swipe_end(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &GestureSwipeEndEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::gesture_swipe_end(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::gesture_swipe_end(x11, seat, data, event),
        }
    }

    fn gesture_pinch_begin(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &GesturePinchBeginEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::gesture_pinch_begin(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::gesture_pinch_begin(x11, seat, data, event),
        }
    }

    fn gesture_pinch_update(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &GesturePinchUpdateEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::gesture_pinch_update(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::gesture_pinch_update(x11, seat, data, event),
        }
    }

    fn gesture_pinch_end(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &GesturePinchEndEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::gesture_pinch_end(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::gesture_pinch_end(x11, seat, data, event),
        }
    }

    fn gesture_hold_begin(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &GestureHoldBeginEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::gesture_hold_begin(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::gesture_hold_begin(x11, seat, data, event),
        }
    }

    fn gesture_hold_end(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &GestureHoldEndEvent) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::gesture_hold_end(wl, seat, data, event),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::gesture_hold_end(x11, seat, data, event),
        }
    }

    fn leave(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, serial: Serial, time: u32) {
        match self {
            PointerFocusTarget::WlSurface(wl) => PointerTarget::leave(wl, seat, data, serial, time),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => PointerTarget::leave(x11, seat, data, serial, time),
        }
    }
}

impl<M: Manager, B: WaylandBackend<M>> TouchTarget<Wayland<M, B>> for PointerFocusTarget {
    fn down(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &TouchDownEvent, seq: Serial) {
        match self {
            PointerFocusTarget::WlSurface(wl) => TouchTarget::down(wl, seat, data, event, seq),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => TouchTarget::down(x11, seat, data, event, seq),
        }
    }

    fn up(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &TouchUpEvent, seq: Serial) {
        match self {
            PointerFocusTarget::WlSurface(wl) => TouchTarget::up(wl, seat, data, event, seq),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => TouchTarget::up(x11, seat, data, event, seq),
        }
    }

    fn motion(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &TouchMotionEvent, seq: Serial) {
        match self {
            PointerFocusTarget::WlSurface(wl) => TouchTarget::motion(wl, seat, data, event, seq),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => TouchTarget::motion(x11, seat, data, event, seq),
        }
    }

    fn frame(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, seq: Serial) {
        match self {
            PointerFocusTarget::WlSurface(wl) => TouchTarget::frame(wl, seat, data, seq),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => TouchTarget::frame(x11, seat, data, seq),
        }
    }

    fn cancel(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, seq: Serial) {
        match self {
            PointerFocusTarget::WlSurface(wl) => TouchTarget::cancel(wl, seat, data, seq),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => TouchTarget::cancel(x11, seat, data, seq),
        }
    }

    fn shape(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &ShapeEvent, seq: Serial) {
        match self {
            PointerFocusTarget::WlSurface(wl) => TouchTarget::shape(wl, seat, data, event, seq),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => TouchTarget::shape(x11, seat, data, event, seq),
        }
    }

    fn orientation(&self, seat: &Seat<Wayland<M, B>>, data: &mut Wayland<M, B>, event: &OrientationEvent, seq: Serial) {
        match self {
            PointerFocusTarget::WlSurface(wl) => TouchTarget::orientation(wl, seat, data, event, seq),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x11) => TouchTarget::orientation(x11, seat, data, event, seq),
        }
    }
}

impl WaylandFocus for PointerFocusTarget {
    #[inline]
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        match self {
            PointerFocusTarget::WlSurface(w) => w.wl_surface(),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x) => x.wl_surface().map(Cow::Owned),
        }
    }

    fn same_client_as(&self, object_id: &ObjectId) -> bool {
        match self {
            PointerFocusTarget::WlSurface(w) => w.same_client_as(object_id),
            #[cfg(feature = "xwayland")]
            PointerFocusTarget::X11Surface(x) => x.same_client_as(object_id)
        }
    }
}

impl WaylandFocus for KeyboardFocusTarget {
    #[inline]
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        match self {
            KeyboardFocusTarget::Window(w) => w.wl_surface(),
            KeyboardFocusTarget::Layer(l) => Some(Cow::Borrowed(l.wl_surface())),
            KeyboardFocusTarget::Popup(p) => Some(Cow::Borrowed(p.wl_surface()))
        }
    }
}

impl From<WlSurface> for PointerFocusTarget {
    #[inline]
    fn from(value: WlSurface) -> Self {
        PointerFocusTarget::WlSurface(value)
    }
}

impl From<&WlSurface> for PointerFocusTarget {
    #[inline]
    fn from(value: &WlSurface) -> Self {
        PointerFocusTarget::from(value.clone())
    }
}

impl From<PopupKind> for PointerFocusTarget {
    #[inline]
    fn from(value: PopupKind) -> Self {
        PointerFocusTarget::from(value.wl_surface())
    }
}

#[cfg(feature = "xwayland")]
impl From<X11Surface> for PointerFocusTarget {
    #[inline]
    fn from(value: X11Surface) -> Self {
        PointerFocusTarget::X11Surface(value)
    }
}

#[cfg(feature = "xwayland")]
impl From<&X11Surface> for PointerFocusTarget {
    #[inline]
    fn from(value: &X11Surface) -> Self {
        PointerFocusTarget::from(value.clone())
    }
}

// impl From<WindowElement> for KeyboardFocusTarget {
//     #[inline]
//     fn from(w: WindowElement) -> Self {
//         KeyboardFocusTarget::Window(w.0.clone())
//     }
// }

impl From<LayerSurface> for KeyboardFocusTarget {
    #[inline]
    fn from(l: LayerSurface) -> Self {
        KeyboardFocusTarget::Layer(l)
    }
}

impl From<PopupKind> for KeyboardFocusTarget {
    #[inline]
    fn from(p: PopupKind) -> Self {
        KeyboardFocusTarget::Popup(p)
    }
}

impl From<KeyboardFocusTarget> for PointerFocusTarget {
    #[inline]
    fn from(value: KeyboardFocusTarget) -> Self {
        match value {
            KeyboardFocusTarget::Window(w) => match w.underlying_surface() {
                WindowSurface::Wayland(w) => PointerFocusTarget::from(w.wl_surface()),
                #[cfg(feature = "xwayland")]
                WindowSurface::X11(s) => PointerFocusTarget::from(s),
            },
            KeyboardFocusTarget::Layer(l) => PointerFocusTarget::from(l.wl_surface()),
            KeyboardFocusTarget::Popup(p) => PointerFocusTarget::from(p.wl_surface()),
        }
    }
}

