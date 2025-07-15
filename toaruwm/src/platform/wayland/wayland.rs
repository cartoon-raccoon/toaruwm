use std::ffi::OsString;
use std::sync::Arc;
use std::os::unix::net::UnixStream;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use thiserror::Error;
use tracing::warn;

use smithay::{
    output::Output, reexports::{
    calloop::{
        generic::Generic, Error as CalloopError, EventLoop, Interest, LoopHandle, 
        LoopSignal, Mode, PostAction
    }, 
    wayland_server::{
        backend::GlobalId,
        protocol::wl_surface::WlSurface, BindError, Display, DisplayHandle
    }
}};
use smithay::backend::{
    session::{
        libseat::Error as SeatError,
    }
};
use smithay::input::{Seat, keyboard::XkbConfig, pointer::PointerHandle};
use smithay::wayland::{
    socket::ListeningSocketSource,
};
use smithay::desktop::{Space, Window, PopupManager};

use super::handlers::{ClientState, WaylandState};
use super::window::{WaylandWindow, Unmapped};
use super::backend::{
    WaylandBackend, WaylandBackendError,
};
use super::render::RedrawState;

use super::{ClientData, Platform, PlatformType};

use crate::platform::{PlatformWindowId, PlatformError, wayland::WaylandOutput};
use crate::config::RuntimeConfig;
use crate::types::{Dict, Rectangle, Logical};
use crate::manager::{Manager, ManagerPlatformInterface};
use crate::Toaru;

/// An identifier corresponding to a Wayland client.
/// 
/// This is the `Client` associated type for [`Wayland`]'s implementation
/// of [`Platform`].
pub type WaylandWindowId = u64;

impl PlatformWindowId for WaylandWindowId {}

/// An implementation of the Wayland platform.
/// 
/// This crate's Wayland functionality is tightly integrated with the `smithay` crate.
/// As such, you will see Smithay types pop up regularly in this module.
#[derive(Debug)]
pub struct Wayland<M, B>
where
    M: Manager<Self> + 'static,
    B: WaylandBackend<M> + 'static
{
    
    /// Our backend.
    pub(super) backend: B,
    /// Our event loop handle.
    pub(super) event_loop: LoopHandle<'static, Self>,
    /// Everything else.
    pub(super) wl: WaylandImpl<M, B>
}

impl<M: Manager<Self>, B: WaylandBackend<M>> Wayland<M, B> {
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
        manager: M, 
        loophandle: LoopHandle<'static, Self>,
        loopsignal: LoopSignal,
    ) -> Result<(Self, Display<Self>), WaylandError>
    {

        let display = Display::new().expect("error initializing Wayland display");

        // create new compositor state and initialize all handlers
        let mut state = WaylandState::new(display.handle());

        let socket = ListeningSocketSource::new_auto()?;
        let socketname = socket.socket_name().to_os_string();

        loophandle.insert_source(socket, move |client, _, wayland| {
            wayland.insert_client(client);
        }).map_err(|e| e.error)?;
        
        // todo: insert wayland socket into environment

        let backend_args = if let Some(args) = backend_args {args} else {Dict::new()};

        let mut seat = state.seat_state.new_wl_seat(&display.handle(), backend.seat_name());

        
        let pointer = seat.add_pointer();
        // todo: get keyboard from config
        seat.add_keyboard(XkbConfig::default(), 200, 25).expect("Could not create default keyboard");

        let mut wl_impl = WaylandImpl {
            manager,
            pointer,
            unmapped: HashMap::new(),
            root_surfaces: HashMap::new(),
            outputs: HashMap::new(),
            global_space: Space::default(),
            popups: PopupManager::default(),
            state,
            seat,
            socketname,
            event_loop: loophandle.clone(),
            display: display.handle(),
            stop_signal: loopsignal
        };

        backend.init(display.handle(), &mut wl_impl, backend_args)?;

        Ok((Self {
            backend,
            event_loop: loophandle,
            wl: wl_impl
        }, display))
    }
}

impl<M: Manager<Self>, B: WaylandBackend<M>> Wayland<M, B> {

    /// Returns a reference to the backend used by `Wayland`.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Returns a mutable reference to the backend used by `Wayland`.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn state(&self) -> &WaylandState<M, B> {
        &self.wl.state
    }

    pub fn state_mut(&mut self) -> &mut WaylandState<M, B> {
        &mut self.wl.state
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
/// loop_handle.insert_source(eventsource,
///     |state: &mut State| state.backend.method_call(state));
/// ```
/// We now have two mutable borrows on `state`, `state.backend` and passing in `state` to the
/// backend method call. The compiler statically checks this isn't allowed, and rightfully blows up
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
/// loop_handle.insert_source(eventsource,
///     |state: &mut State| state.backend.method_call(&mut state.state_impl));
/// ```
/// 
/// Now, we're borrowing from two different fields, and so the compiler allows us to borrow mutably multiple times,
/// since the borrows are on disjoint fields.
/// 
/// ## Usage
/// 
/// You do not need to construct this struct explicitly, it is constructed in [`Wayland::new`],
/// and owned by the `Wayland` struct.
#[derive(Debug)]
pub struct WaylandImpl<M, B>
where
    M: Manager<Wayland<M, B>> + 'static,
    B: WaylandBackend<M> + 'static 
{
    /// The core Toaru struct handling functionality.
    pub(super) manager: M,
    pub(super) pointer: PointerHandle<Wayland<M, B>>,
    /// Unmapped windows.
    pub(super) unmapped: HashMap<WlSurface, Unmapped>,
    /// Cached root surface for every surface, so that we can access it in destroyed()
    /// where the normal get_parent is cleared out.
    pub(super) root_surfaces: HashMap<WlSurface, WlSurface>,
    /// output state.
    pub(super) outputs: HashMap<Output, OutputState>,
    /// The global space that all windows are mapped onto.
    pub(super) global_space: Space<Window>,
    pub(super) popups: PopupManager,
    /// Our smithay state.
    pub(super) state: WaylandState<M, B>,
    pub(super) seat: Seat<Wayland<M, B>>,
    pub(super) socketname: OsString,
    pub(super) event_loop: LoopHandle<'static, Wayland<M, B>>,
    pub(super) display: DisplayHandle,
    pub(super) stop_signal: LoopSignal,
}

impl<M: Manager<Wayland<M, B>> + 'static, B: WaylandBackend<M> + 'static> WaylandImpl<M, B> {
    /// Returns a reference to the internal `Toaru`.
    pub fn manager(&self) -> &M {
        &self.manager
    }

    /// Returns a mutable reference to the internal `Toaru`.
    pub fn toaru_mut(&mut self) -> &mut M {
        &mut self.manager
    }

    /// Creates a new loop handle.
    pub fn loop_handle_new(&self) -> LoopHandle<'static, Wayland<M, B>> {
        self.event_loop.clone()
    }

    /// Adds a new output.
    pub fn add_output(&mut self, output: Output, refresh_interval: Option<Duration>, vrr: bool) {
        // create the global object.
        let global = output.create_global::<Wayland<M, B>>(&self.display);

        // create our output state.
        let outputstate = OutputState {
            global, redraw_state: Default::default(),
        };

        // todo: check against output listed in config, and reconfigure accordingly

        // insert it into our local tracking.
        self.outputs.insert(output.clone(), outputstate);
        // map the output onto our global space.
        let loc = output.current_location();
        self.global_space.map_output(&output, loc);
        // add it to our platform-agnostic state.
        self.manager.add_output(output);
        todo!()
    }

    /// Removes an output from `Wayland`.
    pub fn remove_output(&mut self, output: &Output) {
        todo!()
    }

    /// Repositions all outputs, optionally adding one.
    pub fn reposition_outputs(&mut self, output: Option<&Output>) {

    }

    /// Queue a redraw on all outputs.
    pub fn queue_redraw_all(&mut self) {
        
    }

    /// Queue a redraw on the provided `output`.
    /// 
    /// This is usually called when something in our internal state changes
    /// and needs to be redrawn.
    pub fn queue_redraw(&mut self, output: &Output) {

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

impl PlatformError for WaylandError {}

impl<M: Manager<Self>, B: WaylandBackend<M>> Platform for Wayland<M, B> {
    type WindowId = u64;
    type Window = WaylandWindow;
    type Output = WaylandOutput;
    type Error = WaylandError;

    fn name(&self) -> &str {
        self.backend.name()
    }

    fn platform_type(&self) -> PlatformType {
        PlatformType::Wayland
    }

    fn nested(&self) -> bool {
        self.backend.nested()
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

/// Output-local state.
#[derive(Debug, Clone)]
pub(crate) struct OutputState {
    pub(crate) global: GlobalId,
    pub(crate) redraw_state: RedrawState
}
