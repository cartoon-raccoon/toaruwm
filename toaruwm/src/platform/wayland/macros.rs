//! Re-implementations of `delegate_*` macros from the Smithay and wayland_server crates, to add suppport for bounding generic traits.

/// A helper macro which delegates a set of [`Dispatch`] implementations for a resource to some other type which
/// provides a generic [`Dispatch`] implementation.
///
/// This macro allows more easily delegating smaller parts of the protocol a compositor may wish to handle
/// in a modular fashion.
#[macro_export]
macro_rules! delegate_dispatch {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $dispatch_from:ty : [$interface: ty: $udata: ty] => $dispatch_to: ty) => {
        impl$(< $( $lt $( : $clt $( < $($elt $(= $t1)?),+> )? $(+ $dlt $(< $( $flt $(= $t2)? )+ >)? )* )? ),+ >)? 
        $crate::reexports::wayland_server::Dispatch<$interface, $udata> for $dispatch_from {
            fn request(
                state: &mut Self,
                client: &$crate::reexports::wayland_server::Client,
                resource: &$interface,
                request: <$interface as $crate::reexports::wayland_server::Resource>::Request,
                data: &$udata,
                dhandle: &$crate::reexports::wayland_server::DisplayHandle,
                data_init: &mut $crate::reexports::wayland_server::DataInit<'_, Self>,
            ) {
                <$dispatch_to as $crate::reexports::wayland_server::Dispatch<$interface, $udata, Self>>::request(state, client, resource, request, data, dhandle, data_init)
            }

            fn destroyed(state: &mut Self, client: $crate::reexports::wayland_server::backend::ClientId, resource: &$interface, data: &$udata) {
                <$dispatch_to as $crate::reexports::wayland_server::Dispatch<$interface, $udata, Self>>::destroyed(state, client, resource, data)
            }
        }
    };
}

/// A helper macro which delegates a set of [`GlobalDispatch`] implementations for a resource to some other type which
/// provdes a generic [`GlobalDispatch`] implementation.
///
/// Its usage is similar to the [`delegate_dispatch!()`] macro.
///
/// [`delegate_dispatch!()`]: crate::delegate_dispatch!()
#[macro_export]
macro_rules! delegate_global_dispatch {
    // @<    Generic   :   Trait1  <  TyParam1  (= AssTy)?, ...>     +   TraitN  <
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)?  $dispatch_from:ty : [$interface: ty: $udata: ty] => $dispatch_to: ty) => {
        impl$(< $( $lt $( : $clt $( < $($elt $(= $t1)?),+> )? $(+ $dlt $(< $( $flt $(= $t2)? )+ >)? )* )? ),+ >)? 
        $crate::reexports::wayland_server::GlobalDispatch<$interface, $udata> for $dispatch_from {
            fn bind(
                state: &mut Self,
                dhandle: &$crate::reexports::wayland_server::DisplayHandle,
                client: &$crate::reexports::wayland_server::Client,
                resource: $crate::reexports::wayland_server::New<$interface>,
                global_data: &$udata,
                data_init: &mut $crate::reexports::wayland_server::DataInit<'_, Self>,
            ) {
                <$dispatch_to as $crate::reexports::wayland_server::GlobalDispatch<$interface, $udata, Self>>::bind(state, dhandle, client, resource, global_data, data_init)
            }

            fn can_view(client: $crate::reexports::wayland_server::Client, global_data: &$udata) -> bool {
                <$dispatch_to as $crate::reexports::wayland_server::GlobalDispatch<$interface, $udata, Self>>::can_view(client, global_data)
            }
        }
    };
}

#[macro_export]
macro_rules! delegate_compositor {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_compositor::WlCompositor: ()
        ] => $crate::reexports::smithay::wayland::compositor::CompositorState);
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_subcompositor::WlSubcompositor: ()
        ] => $crate::reexports::smithay::wayland::compositor::CompositorState);

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_compositor::WlCompositor: ()
        ] => $crate::reexports::smithay::wayland::compositor::CompositorState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_surface::WlSurface: $crate::reexports::smithay::wayland::compositor::SurfaceUserData
        ] => $crate::reexports::smithay::wayland::compositor::CompositorState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_region::WlRegion: $crate::reexports::smithay::wayland::compositor::RegionUserData
        ] => $crate::reexports::smithay::wayland::compositor::CompositorState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_callback::WlCallback: ()
        ] => $crate::reexports::smithay::wayland::compositor::CompositorState);

            // WlSubcompositor
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_subcompositor::WlSubcompositor: ()
        ] => $crate::reexports::smithay::wayland::compositor::CompositorState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_subsurface::WlSubsurface: $crate::reexports::smithay::wayland::compositor::SubsurfaceUserData
        ] => $crate::reexports::smithay::wayland::compositor::CompositorState);
    };
}

#[macro_export]
macro_rules! delegate_shm {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_shm::WlShm: ()
        ] => $crate::reexports::smithay::wayland::shm::ShmState);

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_shm::WlShm: ()
        ] => $crate::reexports::smithay::wayland::shm::ShmState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_shm_pool::WlShmPool: $crate::reexports::smithay::wayland::shm::ShmPoolUserData
        ] => $crate::reexports::smithay::wayland::shm::ShmState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_buffer::WlBuffer: $crate::reexports::smithay::wayland::shm::ShmBufferUserData
        ] => $crate::reexports::smithay::wayland::shm::ShmState);
    };
}

/// Macro to delegate implementation of the linux dmabuf to [`DmabufState`].
///
/// You must also implement [`DmabufHandler`] to use this.
#[macro_export]
macro_rules! delegate_dmabuf {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        type __ZwpLinuxDmabufV1 =
            $crate::reexports::wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1;
        type __ZwpLinuxBufferParamsV1 =
            $crate::reexports::wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1;
        type __ZwpLinuxDmabufFeedbackv1 =
            $crate::reexports::wayland_protocols::wp::linux_dmabuf::zv1::server::zwp_linux_dmabuf_feedback_v1::ZwpLinuxDmabufFeedbackV1;

        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            __ZwpLinuxDmabufV1: $crate::reexports::smithay::wayland::dmabuf::DmabufGlobalData
        ] => $crate::reexports::smithay::wayland::dmabuf::DmabufState);

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            __ZwpLinuxDmabufV1: $crate::reexports::smithay::wayland::dmabuf::DmabufData
        ] => $crate::reexports::smithay::wayland::dmabuf::DmabufState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            __ZwpLinuxBufferParamsV1: $crate::reexports::smithay::wayland::dmabuf::DmabufParamsData
        ] => $crate::reexports::smithay::wayland::dmabuf::DmabufState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer: $crate::reexports::smithay::backend::allocator::dmabuf::Dmabuf
        ] => $crate::reexports::smithay::wayland::dmabuf::DmabufState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            __ZwpLinuxDmabufFeedbackv1: $crate::reexports::smithay::wayland::dmabuf::DmabufFeedbackData
        ] => $crate::reexports::smithay::wayland::dmabuf::DmabufState);

    };
}

#[macro_export]
macro_rules! delegate_output {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_output::WlOutput: $crate::reexports::smithay::wayland::output::WlOutputData
        ] => $crate::reexports::smithay::wayland::output::OutputManagerState);
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::xdg_output::zv1::server::zxdg_output_manager_v1::ZxdgOutputManagerV1: ()
        ] => $crate::reexports::smithay::wayland::output::OutputManagerState);

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_output::WlOutput: $crate::reexports::smithay::wayland::output::OutputUserData
        ] => $crate::reexports::smithay::wayland::output::OutputManagerState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::xdg_output::zv1::server::zxdg_output_v1::ZxdgOutputV1: $crate::reexports::smithay::wayland::output::XdgOutputUserData
        ] => $crate::reexports::smithay::wayland::output::OutputManagerState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::xdg_output::zv1::server::zxdg_output_manager_v1::ZxdgOutputManagerV1: ()
        ] => $crate::reexports::smithay::wayland::output::OutputManagerState);
    };
}

#[macro_export]
macro_rules! delegate_seat {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_seat::WlSeat: $crate::reexports::smithay::wayland::seat::SeatGlobalData<$ty>
        ] => $crate::reexports::smithay::input::SeatState<$ty>);

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_seat::WlSeat: $crate::reexports::smithay::wayland::seat::SeatUserData<$ty>
        ] => $crate::reexports::smithay::input::SeatState<$ty>);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_pointer::WlPointer: $crate::reexports::smithay::wayland::seat::PointerUserData<$ty>
        ] => $crate::reexports::smithay::input::SeatState<$ty>);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_keyboard::WlKeyboard: $crate::reexports::smithay::wayland::seat::KeyboardUserData<$ty>
        ] => $crate::reexports::smithay::input::SeatState<$ty>);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_touch::WlTouch: $crate::reexports::smithay::wayland::seat::TouchUserData<$ty>
        ] => $crate::reexports::smithay::input::SeatState<$ty>);
    };
}

#[macro_export]
macro_rules! delegate_pointer_gestures {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::wp::pointer_gestures::zv1::server::zwp_pointer_gestures_v1::ZwpPointerGesturesV1: ()
        ] => $crate::reexports::smithay::wayland::pointer_gestures::PointerGesturesState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::wp::pointer_gestures::zv1::server::zwp_pointer_gestures_v1::ZwpPointerGesturesV1: ()
        ] => $crate::reexports::smithay::wayland::pointer_gestures::PointerGesturesState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::wp::pointer_gestures::zv1::server::zwp_pointer_gesture_swipe_v1::ZwpPointerGestureSwipeV1: 
                $crate::reexports::smithay::wayland::pointer_gestures::PointerGestureUserData<Self>
        ] => $crate::reexports::smithay::wayland::pointer_gestures::PointerGesturesState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::wp::pointer_gestures::zv1::server::zwp_pointer_gesture_pinch_v1::ZwpPointerGesturePinchV1: 
                $crate::reexports::smithay::wayland::pointer_gestures::PointerGestureUserData<Self>
        ] => $crate::reexports::smithay::wayland::pointer_gestures::PointerGesturesState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::wp::pointer_gestures::zv1::server::zwp_pointer_gesture_hold_v1::ZwpPointerGestureHoldV1: 
                $crate::reexports::smithay::wayland::pointer_gestures::PointerGestureUserData<Self>
        ] => $crate::reexports::smithay::wayland::pointer_gestures::PointerGesturesState);
    };
}

#[macro_export]
macro_rules! delegate_relative_pointer {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::wp::relative_pointer::zv1::server::zwp_relative_pointer_manager_v1::ZwpRelativePointerManagerV1: ()
        ] => $crate::reexports::smithay::wayland::relative_pointer::RelativePointerManagerState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::wp::relative_pointer::zv1::server::zwp_relative_pointer_manager_v1::ZwpRelativePointerManagerV1: ()
        ] => $crate::reexports::smithay::wayland::relative_pointer::RelativePointerManagerState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::wp::relative_pointer::zv1::server::zwp_relative_pointer_v1::ZwpRelativePointerV1: 
                $crate::reexports::smithay::wayland::relative_pointer::RelativePointerUserData<Self>
        ] => $crate::reexports::smithay::wayland::relative_pointer::RelativePointerManagerState);
    };
}

#[macro_export]
macro_rules! delegate_data_device {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_data_device_manager::WlDataDeviceManager: ()
        ] => $crate::reexports::smithay::wayland::selection::data_device::DataDeviceState);

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_data_device_manager::WlDataDeviceManager: ()
        ] => $crate::reexports::smithay::wayland::selection::data_device::DataDeviceState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_data_device::WlDataDevice: 
                $crate::reexports::smithay::wayland::selection::data_device::DataDeviceUserData
        ] => $crate::reexports::smithay::wayland::selection::data_device::DataDeviceState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_data_source::WlDataSource: 
                $crate::reexports::smithay::wayland::selection::data_device::DataSourceUserData
        ] => $crate::reexports::smithay::wayland::selection::data_device::DataDeviceState);
    };
}

#[macro_export]
macro_rules! delegate_xdg_shell {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::shell::server::xdg_wm_base::XdgWmBase: ()
        ] => $crate::reexports::smithay::wayland::shell::xdg::XdgShellState);

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::shell::server::xdg_wm_base::XdgWmBase:
                $crate::reexports::smithay::wayland::shell::xdg::XdgWmBaseUserData
        ] => $crate::reexports::smithay::wayland::shell::xdg::XdgShellState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::shell::server::xdg_positioner::XdgPositioner:
                $crate::reexports::smithay::wayland::shell::xdg::XdgPositionerUserData
        ] => $crate::reexports::smithay::wayland::shell::xdg::XdgShellState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::shell::server::xdg_popup::XdgPopup:
                $crate::reexports::smithay::wayland::shell::xdg::XdgShellSurfaceUserData
        ] => $crate::reexports::smithay::wayland::shell::xdg::XdgShellState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::shell::server::xdg_surface::XdgSurface:
                $crate::reexports::smithay::wayland::shell::xdg::XdgSurfaceUserData
        ] => $crate::reexports::smithay::wayland::shell::xdg::XdgShellState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::XdgToplevel:
                $crate::reexports::smithay::wayland::shell::xdg::XdgShellSurfaceUserData
        ] => $crate::reexports::smithay::wayland::shell::xdg::XdgShellState);
    };
}

/// Macro to delegate implementation of the xdg decoration to [`XdgDecorationState`].
///
/// You must also implement [`XdgDecorationHandler`] to use this.
#[macro_export]
macro_rules! delegate_xdg_decoration {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_decoration_manager_v1::ZxdgDecorationManagerV1: 
                $crate::reexports::smithay::wayland::shell::xdg::decoration::XdgDecorationManagerGlobalData
        ] => $crate::reexports::smithay::wayland::shell::xdg::decoration::XdgDecorationState);

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_decoration_manager_v1::ZxdgDecorationManagerV1: ()
        ] => $crate::reexports::smithay::wayland::shell::xdg::decoration::XdgDecorationState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::ZxdgToplevelDecorationV1: 
                $crate::reexports::smithay::wayland::shell::xdg::ToplevelSurface
        ] => $crate::reexports::smithay::wayland::shell::xdg::decoration::XdgDecorationState);
    };
}

/// Macro to delegate implementation of the xdg dialog to [`XdgDialogState`].
///
/// You must also implement [`XdgDialogHandler`] to use this.
#[macro_export]
macro_rules! delegate_xdg_dialog {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::dialog::v1::server::xdg_wm_dialog_v1::XdgWmDialogV1: ()
        ] => $crate::reexports::smithay::wayland::shell::xdg::dialog::XdgDialogState);

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::dialog::v1::server::xdg_wm_dialog_v1::XdgWmDialogV1: ()
        ] => $crate::reexports::smithay::wayland::shell::xdg::dialog::XdgDialogState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_protocols::xdg::dialog::v1::server::xdg_dialog_v1::XdgDialogV1:
                $crate::reexports::smithay::wayland::shell::xdg::ToplevelSurface
        ] => $crate::reexports::smithay::wayland::shell::xdg::dialog::XdgDialogState);
    };
}

/// Macro to delegate implementation of the xdg foreign to [`XdgForeignState`].
///
/// You must also implement [`XdgForeignHandler`] and
/// [`XdgShellHandler`](crate::wayland::shell::xdg::XdgShellHandler) to use this.
#[macro_export]
macro_rules! delegate_xdg_foreign {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        type __ZxdgExporterV2 =
            $crate::reexports::wayland_protocols::xdg::foreign::zv2::server::zxdg_exporter_v2::ZxdgExporterV2;
        type __ZxdgImporterV2 =
            $crate::reexports::wayland_protocols::xdg::foreign::zv2::server::zxdg_importer_v2::ZxdgImporterV2;

        type __ZxdgExportedV2 =
            $crate::reexports::wayland_protocols::xdg::foreign::zv2::server::zxdg_exported_v2::ZxdgExportedV2;
        type __ZxdgImportedV2 =
            $crate::reexports::wayland_protocols::xdg::foreign::zv2::server::zxdg_imported_v2::ZxdgImportedV2;

        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty:
            [
                __ZxdgExporterV2: ()
            ] => $crate::reexports::smithay::wayland::xdg_foreign::XdgForeignState
        );
        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty:
            [
                __ZxdgImporterV2: ()
            ] => $crate::reexports::smithay::wayland::xdg_foreign::XdgForeignState
        );

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty:
            [
                __ZxdgExporterV2: ()
            ] => $crate::reexports::smithay::wayland::xdg_foreign::XdgForeignState
        );
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty:
            [
                __ZxdgImporterV2: ()
            ] => $crate::reexports::smithay::wayland::xdg_foreign::XdgForeignState
        );

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty:
            [
                __ZxdgExportedV2: $crate::reexports::smithay::wayland::xdg_foreign::XdgExportedUserData
            ] => $crate::reexports::smithay::wayland::xdg_foreign::XdgForeignState
        );
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty:
            [
                __ZxdgImportedV2: $crate::reexports::smithay::wayland::xdg_foreign::XdgImportedUserData
            ] => $crate::reexports::smithay::wayland::xdg_foreign::XdgForeignState
        );
    };
}

/// Macro to delegate implementation of wlr layer shell to [`WlrLayerShellState`].
///
/// You must also implement [`WlrLayerShellHandler`] to use this.
#[macro_export]
macro_rules! delegate_layer_shell {
    ($(@< $( $lt:tt $( : $clt:tt $(< $($elt:ty $(= $t1:ty)?),+ >)? $(+ $dlt:tt $(< $( $flt:tt $(= $t2:ty)? ),+ >)? )* )? ),+ >)? $ty: ty) => {
        type __ZwlrLayerShellV1 =
            $crate::reexports::smithay::reexports::wayland_protocols_wlr::layer_shell::v1::server::zwlr_layer_shell_v1::ZwlrLayerShellV1;
        type __ZwlrLayerShellSurfaceV1 =
            $crate::reexports::smithay::reexports::wayland_protocols_wlr::layer_shell::v1::server::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1;

        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            __ZwlrLayerShellV1: ()
        ] => $crate::reexports::smithay::wayland::shell::wlr_layer::WlrLayerShellState);
        $crate::delegate_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            __ZwlrLayerShellSurfaceV1: $crate::reexports::smithay::wayland::shell::wlr_layer::WlrLayerSurfaceUserData
        ] => $crate::reexports::smithay::wayland::shell::wlr_layer::WlrLayerShellState);

        $crate::delegate_global_dispatch!($(@< $( $lt $( : $clt $(< $($elt $(= $t1)?),+ >)? $(+ $dlt $(<$($flt $(= $t2)?)*>)? )* )? ),+ >)? $ty: [
            __ZwlrLayerShellV1: $crate::reexports::smithay::wayland::shell::wlr_layer::WlrLayerShellGlobalData
        ] => $crate::reexports::smithay::wayland::shell::wlr_layer::WlrLayerShellState);
    };
}

