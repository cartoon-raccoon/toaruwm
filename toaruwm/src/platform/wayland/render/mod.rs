//! Backend-agnostic rendering functionality.

mod macros;

use smithay::reexports::{calloop::RegistrationToken};
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::backend::renderer::gles::{GlesFrame, GlesRenderer, GlesTexture, GlesError};
use smithay::backend::renderer::{
    Bind, ExportMem, ImportAll, ImportMem, Offscreen, Renderer, RendererSuper, Texture,
};

/// A macro to aggregate 
#[macro_export]
macro_rules! toaru_render_elements {
    {$(#[$attr:meta])+ $vis:vis $name:ident<$renderer:ident> => $($tail:tt)*} => {
        
    };

    {$(#[$attr:meta])+ $vis:vis $name:ident => $($tail:tt)*} => {

    }
}

/// A marker trait marking all the trait requirements for a renderer object to be used
/// within Toaru.
// Shamelessly stolen from Niri.
pub trait ToaruRenderer:
    ImportAll
    + ImportMem
    + ExportMem
    + Bind<Dmabuf>
    + Offscreen<GlesTexture>
    + Renderer<TextureId = Self::ToaruTextureId, Error = Self::ToaruRenderError>
{
    type ToaruTextureId: Texture + Clone + Send + 'static;
    type ToaruRenderError: std::error::Error
        + Send
        + Sync
        + From<<GlesRenderer as RendererSuper>::Error>
        + 'static;
}

impl ToaruRenderer for GlesRenderer {
    type ToaruTextureId = GlesTexture;
    type ToaruRenderError = GlesError;
}

pub trait AsGlesRenderer {
    fn as_gles_renderer(&mut self) -> &mut GlesRenderer;
}

impl AsGlesRenderer for GlesRenderer {
    fn as_gles_renderer(&mut self) -> &mut GlesRenderer {
        self
    }
}

pub trait AsGlesFrame<'frame, 'buffer>
where
    Self: 'frame,
{
    fn as_gles_frame(&mut self) -> &mut GlesFrame<'frame, 'buffer>;
}

impl<'frame, 'buffer> AsGlesFrame<'frame, 'buffer> for GlesFrame<'frame, 'buffer> {
    fn as_gles_frame(&mut self) -> &mut GlesFrame<'frame, 'buffer> {
        self
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

/// The state of an Output.
#[derive(Debug, Default, Copy, Clone)]
pub enum RedrawState {
    /// This output is idle.
    #[default]
    Idle,
    /// A redraw has been queued.
    Queued,
    /// A frame has been submitted and we are waiting for it to be presented.
    WaitingForVBlank {redraw_needed: bool},
    /// Nothing was submitted and we made a timer to fire at the estimated VBlank.
    WaitingForEstimatedVBlank(RegistrationToken),
    /// A redraw is queued on top of the above.
    WaitingForEstimatedVBlankAndQueued(RegistrationToken),
}

impl RedrawState {
    /// Step to the next state in the state machine for each variant.
    pub fn queue_redraw(self) -> Self {
        match self {
            RedrawState::Idle => RedrawState::Queued,
            RedrawState::WaitingForEstimatedVBlank(token) => {
                RedrawState::WaitingForEstimatedVBlankAndQueued(token)
            }
            // redraw already queued, return self
            value @ (RedrawState::Queued | RedrawState::WaitingForEstimatedVBlankAndQueued(_)) => {
                value
            }
            RedrawState::WaitingForVBlank { .. } => 
                RedrawState::WaitingForVBlank { redraw_needed: true }
        }
    }
}

