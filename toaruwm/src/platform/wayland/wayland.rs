use std::ffi::OsString;
use std::sync::Arc;
use std::os::unix::net::UnixStream;
use std::collections::HashMap;
use std::time::Duration;

use thiserror::Error;
use tracing::warn;

use smithay::{output::Output, reexports::{
    calloop::{
        generic::Generic, Error as CalloopError, EventLoop, Interest, LoopHandle, LoopSignal, Mode, PostAction
    }, 
    wayland_server::{
        protocol::wl_surface::WlSurface, BindError, Display, DisplayHandle
    }
}};
use smithay::backend::{
    input::{InputEvent, InputBackend},
    session::{
        libseat::Error as SeatError,
    }
};
use smithay::input::Seat;
use smithay::wayland::{
    socket::ListeningSocketSource,
};
use smithay::desktop::{Space, Window};

use super::state::{ClientState, WlState};
use super::window::{Unmapped};
use super::backend::{
    WaylandBackend, WaylandBackendInit, WaylandBackendError,
};

use super::{ClientData, Platform, PlatformType};

use crate::platform::{PlatformWindowId, wayland::WaylandOutput};
use crate::config::RuntimeConfig;
use crate::types::{Dict, Rectangle, Logical};

use crate::Toaru;

/// An identifier corresponding to a Wayland client.
/// 
/// This is the `Client` associated type for [`Wayland`]'s implementation
/// of [`Platform`].
pub type WaylandClientId = u64;

impl PlatformWindowId for WaylandClientId {}

/// An implementation of the Wayland platform.
/// 
/// This crate's Wayland functionality is tightly integrated with the `smithay` crate.
/// As such, you will see Smithay types pop up regularly in this module.
#[derive(Debug)]
pub struct Wayland<C, B>
where
    C: RuntimeConfig + 'static,
    B: WaylandBackend + 'static
{
    
    /// Our backend.
    pub(super) backend: B,
    /// Our event loop handle.
    pub(super) event_loop: LoopHandle<'static, Self>,
    /// Everything else.
    pub(super) wl_impl: WaylandImpl<C, B>
}

impl<C: RuntimeConfig, B: WaylandBackend> Wayland<C, B> {
    /// Creates a new Wayland compositor, and runs [`init`][1] on the given `backend`.
    /// On success, returns a (Self, Display) tuple, and the
    /// display should be ultimately passed into the run() method.
    /// 
    /// ## Parameters
    /// 
    /// - `backend`: The `WaylandBackend` implementation that `Wayland` will use.
    /// - `backend_args`: Any additional arguments that `backend` will use to initialize
    /// itself, returned by the constructor.
    /// - a `calloop` event loop handle.
    /// - a `calloop` loop signal.
    /// 
    /// [1]: super::backend::WaylandBackendInit::init
    pub fn new(
        mut backend: B,
        backend_args: Option<Dict>,
        toaru: Toaru<Self, C>, 
        loophandle: LoopHandle<'static, Self>,
        loopsignal: LoopSignal,
    ) -> Result<(Self, Display<Self>), WaylandError>
    where
        B: WaylandBackendInit<C>
    {

        let display = Display::new().expect("error initializing Wayland display");

        // create new compositor state and initialize all handlers
        let mut state = WlState::new(display.handle());

        let socket = ListeningSocketSource::new_auto()?;
        let socketname = socket.socket_name().to_os_string();

        loophandle.insert_source(socket, move |client, _, wayland| {
            wayland.insert_client(client);
        }).map_err(|e| e.error)?;
        
        // todo: insert wayland socket into environment

        let backend_args = if let Some(args) = backend_args {args} else {Dict::new()};

        let mut seat = state.seat_state.new_wl_seat(&display.handle(), backend.seat_name());

        // todo: add keyboard

        seat.add_pointer();

        let mut wl_impl = WaylandImpl {
            toaru,
            unmapped: HashMap::new(),
            root_surfaces: HashMap::new(),
            global_space: Space::default(),
            state,
            seat,
            socketname,
            event_loop: loophandle.clone(),
            stop_signal: loopsignal
        };

        backend.init(display.handle(), &mut wl_impl, backend_args)?;

        Ok((Self {
            backend,
            event_loop: loophandle,
            wl_impl
        }, display))
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn state(&self) -> &WlState<C, B> {
        &self.wl_impl.state
    }

    pub fn state_mut(&mut self) -> &mut WlState<C, B> {
        &mut self.wl_impl.state
    }

    pub fn get_loop_handle(&self) -> &LoopHandle<'static, Self> {
        &self.event_loop
    }

    pub fn new_loop_handle(&self) -> LoopHandle<'static, Self> {
        self.event_loop.clone()
    }

    pub fn get_display_handle(&self) -> &DisplayHandle {
        &self.state().display_handle
    }

    pub fn new_display_handle(&self) -> DisplayHandle {
        self.state().display_handle.clone()
    }


    /// Run the event loop.
    pub fn run(&mut self, 
        display: Display<Self>, 
        mut eventloop: EventLoop<'_, Self>
    ) -> Result<(), WaylandError> {

        let display_src = Generic::new(display, Interest::READ, Mode::Level);

        let loophandle = eventloop.handle();

        // insert
        loophandle.insert_source(display_src, |_, display, state| {
            // SAFETY: we don't drop the display.
            if let Err(e) = unsafe { display.get_mut().dispatch_clients(state) } {
                warn!("error while dispatching clients: {e}");
            }
            Ok(PostAction::Continue)
        }).expect("");

        #[cfg(feature = "xwayland")]
        self.start_xwayland();
        
        eventloop.run(None, self, |_| {}).unwrap();
        Ok(())
    }

    #[cfg(feature = "xwayland")]
    pub(crate) fn start_xwayland(&mut self) {

    }

    /// Create the elements for rendering.
    pub fn render_elements(&mut self) {

    }

    pub(crate) fn handle_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {

    }

    /// General client insertion.
    /// 
    /// Clients only get added to Toaru when they are XDG.
    fn insert_client(&mut self, client: UnixStream) {
        let data = ClientState {
            compositor_state: Default::default(),
            restricted: false,
            credentials_unknown: false
        };
        if let Err(e) = self.state_mut().display_handle.insert_client(client, Arc::new(data)) {
            warn!("error while registering new client: {e}");
        }
    }
}

/// [`Wayland`], without the backend.
/// 
/// `WaylandImpl` stores the core non-backend state of the `Platform` that owns it. This allows
/// us to pass in mutable references to it, into backend method calls, so we can still access all
/// our state from the backend method call.
/// 
/// ## A motivating example
/// 
/// When constructing your backend, there will be many instances where you need to insert callbacks
/// into your event loop through your loop handle. These callbacks are passed a mutable reference to
/// your state as an argument, so you now have full access to your state and all its fields.
/// 
/// Imagine your state was laid out like this:
/// 
/// ```no_run
/// pub struct State {
///     backend: Backend, // your struct which implements WaylandBackend
///     /* .. your other state fields ... */
/// }
/// 
/// ...
/// 
/// impl Backend {
///     pub fn method_call(&mut self, state: &mut State) { ... }
/// }
/// 
/// ```
/// and so when inserting your callback, it looks like this:
/// 
/// ```no_run
/// loop_handle.insert_source(eventsource, |state: &mut State| state.backend.method_call(state));
/// ```
/// Oh no! We now have two mutable borrows on `state`, `state.backend` and passing in `state` to the
/// backend method call! The compiler statically checks this will not work, and rightfully blows up
/// in our face.
/// 
/// Now imagine your state was laid out like this:
/// 
/// ```no_run
/// pub struct StateImpl {
///     /* ... state fields ... */
/// }
/// pub struct State {
///     backend: Backend,
///     // State fields are stored under one struct
///     state_impl: StateImpl,
/// }
/// 
/// impl Backend {
///     pub fn method_call(&mut self, state_impl: &mut StateImpl) { ... }
/// }
/// ```
/// 
/// And when we insert our callback, we can do this:
/// 
/// ```no_run
/// loop_handle.insert_source(eventsource, |state: &mut State| state.backend.method_call(&mut state.state_impl));
/// ```
/// 
/// Now, we're borrowing from two different fields, and so the compiler allows us to borrow mutably multiple times,
/// since the borrows are on disjoint fields.
/// 
/// ---
/// 
/// You do not need to construct this struct explicitly, it is constructed in [`Wayland::new`],
/// and owned by the `Wayland` struct.
#[derive(Debug)]
pub struct WaylandImpl<C, B>
where
    C: RuntimeConfig + 'static,
    B: WaylandBackend + 'static 
{
    /// The core Toaru struct handling functionality.
    pub(super) toaru: Toaru<Wayland<C, B>, C>,
    /// Unmapped windows.
    pub(super) unmapped: HashMap<WlSurface, Unmapped>,
    /// Cached root surface for every surface, so that we can access it in destroyed()
    /// where the normal get_parent is cleared out.
    pub(super) root_surfaces: HashMap<WlSurface, WlSurface>,
    /// The global space that all windows are mapped onto.
    pub(super) global_space: Space<Window>,
    /// Our smithay state.
    pub(super) state: WlState<C, B>,
    pub(super) seat: Seat<Wayland<C, B>>,
    pub(super) socketname: OsString,
    pub(super) event_loop: LoopHandle<'static, Wayland<C, B>>,
    pub(super) stop_signal: LoopSignal,
}

impl<C: RuntimeConfig + 'static, B: WaylandBackend + 'static> WaylandImpl<C, B> {

    pub fn loop_handle_new(&self) -> LoopHandle<'static, Wayland<C, B>> {
        self.event_loop.clone()
    }

    pub fn add_output(&mut self, output: Output, refresh_interval: Duration, vrr: bool) {
        todo!()
    }

    pub fn remove_output(&mut self, output: &Output) {
        todo!()
    }
}


#[derive(Error, Debug)]
pub enum WaylandError {
    #[error("unable to establish seat: {0}")]
    SessionErr(#[from] SeatError),
    #[error("udev failure: {0}")]
    UdevErr(String),
    #[error("backend error: {0}")]
    BackendError(Box<dyn WaylandBackendError>),
    #[error(transparent)]
    EventLoopErr(#[from] CalloopError),
    #[error(transparent)]
    SocketBindErr(#[from] BindError),
}

impl<E: WaylandBackendError + 'static> From<E> for WaylandError {
    fn from(e: E) -> WaylandError {
        WaylandError::BackendError(Box::new(e))
    }
}

impl<C: RuntimeConfig, B: WaylandBackend> Platform for Wayland<C, B> {
    type WindowId = u64;
    type Output = WaylandOutput;
    type Error = WaylandError;

    fn name(&self) -> &str {
        self.backend.name()
    }

    fn platform_type(&self) -> PlatformType {
        PlatformType::Wayland
    }

    fn nested(&self) -> bool {
        self.backend.name() == "winit"
    }

    fn all_outputs(&self) -> Result<&[Self::Output], WaylandError> {
        todo!()
    }

    fn query_tree(&self, client: u64) -> Result<Rectangle<i32, Logical>, WaylandError> {
        todo!()
    }

    fn query_pointer(&self) {
        todo!()
    }

    fn query_window_data(&self, clid: u64) -> Result<ClientData, WaylandError> {
        todo!()
    }
}
