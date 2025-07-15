
use smithay::desktop::{
    Window,
};
use smithay::wayland::{
    xdg_activation::XdgActivationTokenData,
};
use smithay::output::Output;

use super::WINDOW_ID_GENERATOR;

use crate::types::{Rectangle, Logical};

/// An unmapped window.
#[derive(Debug, Clone)]
pub struct Unmapped {
    id: u64,
    pub(crate) window: Window,
    configure_state: ConfigureState,
    activation_token_data: Option<XdgActivationTokenData>,
}

impl Unmapped {
    /// Creates a new Unmapped from a Smithay Window.
    pub fn new(window: Window) -> Self {
        Self {
            id: WINDOW_ID_GENERATOR.next(),
            window,
            configure_state: ConfigureState::Unconfigured { wants_fullscreen: None },
            activation_token_data: None,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    /// The unmapped needs to be configured.
    pub fn needs_initial_configure(&self) -> bool {
        matches!(self.configure_state, ConfigureState::Unconfigured { wants_fullscreen: _ })
    }
}

/// The current configuration state of the window.
#[derive(Debug, Clone)]
pub enum ConfigureState {
    /// The window has not been initially configured.
    Unconfigured {
        /// Whether the window wants to be fullscreened, and on which output, if any.
        wants_fullscreen: Option<Option<Output>>
    },
    /// The window has been configured and is waiting to be mapped.
    Configured {
        /// The configured geometry of the window.
        geometry: Rectangle<i32, Logical>,

        /// The floating geometry of the window.
        floating_geometry: Option<Rectangle<i32, Logical>>,

        /// Output to open this window on.
        ///
        /// This can be `None` in cases like:
        ///
        /// - There are no outputs connected.
        /// - This is a dialog with a parent, and there was no explicit output set, so this dialog
        ///   should fetch the parent's current output again upon mapping.
        output: Option<Output>,

        /// The workspace to open this window on, if any.
        workspace: Option<String>,
    }
}