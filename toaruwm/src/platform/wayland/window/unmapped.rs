
use smithay::desktop::{
    Window,
};
use smithay::wayland::{
    xdg_activation::XdgActivationTokenData,
};
use smithay::output::Output;

/// An unmapped window.
#[derive(Debug)]
pub struct Unmapped {
    window: Window,
    configure_state: ConfigureState,
    activation_token_data: Option<XdgActivationTokenData>,
}

impl Unmapped {
    /// Creates a new Unmapped from a Smithay Window.
    pub fn new(window: Window) -> Self {
        Self {
            window,
            configure_state: ConfigureState::Unconfigured { wants_fullscreen: None },
            activation_token_data: None,
        }
    }

    /// The unmapped needs to be configured.
    pub fn needs_initial_configure(&self) -> bool {
        matches!(self.configure_state, ConfigureState::Unconfigured { wants_fullscreen: _ })
    }
}

/// The current configuration state of the window.
#[derive(Debug)]
pub enum ConfigureState {
    /// The window has not been initially configured.
    Unconfigured {
        /// Whether the window wants to be fullscreened, and on which output, if any.
        wants_fullscreen: Option<Option<Output>>
    },
    Configured {

    }
}