use std::ffi::OsString;
use std::sync::Arc;
use std::os::unix::net::UnixStream;

use thiserror::Error;
use tracing::warn;

use smithay::reexports::{
    calloop::{
        generic::Generic, Error as CalloopError,
        EventLoop, Interest, LoopHandle, Mode, PostAction, LoopSignal,
    }, 
    wayland_server::{
        backend::{ClientId as WlsClientId, InvalidId},
        Display, DisplayHandle, BindError,
    }
};
use smithay::backend::{
    input::{InputEvent, InputBackend},
    session::{
        libseat::Error as SeatError,
    }
};
use smithay::wayland::{
    socket::ListeningSocketSource,
};
use smithay::desktop::{Space, Window};

use super::state::{ClientState, WlState};

use super::backend::{
    WaylandBackend, WaylandBackendError,
};

use super::{ClientData, Platform, PlatformType};

use crate::{core::types::ClientId, platform::Output};
use crate::manager::state::{RuntimeConfig};
use crate::types::{Dict, Rectangle, Logical};

use crate::Toaru;

/// An identifier corresponding to a Wayland client.
/// 
/// This is the `Client` associated type for [`Wayland`]'s implementation
/// of [`Platform`].
pub type WaylandClientId = WlsClientId;

impl ClientId for WaylandClientId {}

/// An implementation of the Wayland platform.
/// 
/// This crate's Wayland functionality is tightly integrated with the `smithay` crate.
/// As such, you will see Smithay types pop up regularly in this module.
#[derive(Debug)]
pub struct Wayland<C, B>
where
    C: RuntimeConfig,
    B: WaylandBackend
{
    pub(super) toaru: Toaru<Wayland<C, B>, C>,
    pub(super) global_space: Space<Window>,
    pub(super) state: WlState,
    pub(super) backend: B,
    pub(super) socketname: OsString,
    pub(super) event_loop: LoopHandle<'static, Wayland<C, B>>,
    pub(super) stop_signal: LoopSignal,
}

impl<C: RuntimeConfig, B: WaylandBackend> Wayland<C, B> {
    /// Creates a new Wayland compositor, and runs [`init()`][1] on the given `backend`.
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
    /// [1]: backend::WaylandBackend::init
    pub fn new(
        mut backend: B,
        backend_args: Option<Dict>,
        toaru: Toaru<Self, C>, 
        loophandle: LoopHandle<'static, Self>,
        loopsignal: LoopSignal,
    ) -> Result<(Self, Display<Self>), WaylandError> {

        let display = Display::new().expect("error initializing Wayland display");

        // create new compositor state and initialize all handlers
        let mut state = WlState::new::<C, B>(display.handle());

        let socket = ListeningSocketSource::new_auto()?;
        let socketname = socket.socket_name().to_os_string();

        loophandle.insert_source(socket, move |client, _, wayland| {
            wayland.insert_client(client);
        }).map_err(|e| e.error)?;
        
        // todo: insert wayland socket into environment

        let backend_args = if let Some(args) = backend_args {args} else {Dict::new()};

        backend.init(loophandle.clone(), display.handle(), &mut state, backend_args)?;

        Ok((Self {
            toaru,
            global_space: Space::default(),
            state,
            backend,
            socketname,
            event_loop: loophandle,
            stop_signal: loopsignal
        }, display))
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn state(&self) -> &WlState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut WlState {
        &mut self.state
    }

    pub fn get_loop_handle(&self) -> &LoopHandle<'static, Self> {
        &self.event_loop
    }

    pub fn new_loop_handle(&self) -> LoopHandle<'static, Self> {
        self.event_loop.clone()
    }

    pub fn get_display_handle(&self) -> &DisplayHandle {
        &self.state.display_handle
    }

    pub fn new_display_handle(&self) -> DisplayHandle {
        self.state.display_handle.clone()
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

    /// Create
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
        if let Err(e) = self.state.display_handle.insert_client(client, Arc::new(data)) {
            warn!("error while registering new client: {e}");
        }
    }
}

/// The result of a rendering operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderResult {
    /// The frame was successfully rendered and submitted.
    Submitted,
    /// Rendering was successful but there was no damage.
    NoDamage,
    /// The frame was not rendered and submitted.
    Skipped
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
    #[error("no such client: {1:?}")]
    NoSuchClient(#[source] InvalidId, WlsClientId),
}

impl<E: WaylandBackendError + 'static> From<E> for WaylandError {
    fn from(e: E) -> WaylandError {
        WaylandError::BackendError(Box::new(e))
    }
}

impl<C: RuntimeConfig, B: WaylandBackend> Platform for Wayland<C, B> {
    type Client = WlsClientId;
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

    fn all_outputs(&self) -> Result<&[Output], WaylandError> {
        todo!()
    }

    fn query_tree(&self, client: &WlsClientId) -> Result<Rectangle<Logical>, WaylandError> {
        todo!()
    }

    fn query_pointer(&self) {
        todo!()
    }

    fn query_client_data(&self, clid: &WlsClientId) -> Result<ClientData, WaylandError> {
        todo!()
    }
}
