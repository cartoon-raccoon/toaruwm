use std::fmt;

use crate::core::{Screen, Workspace};
use crate::types::Geometry;
use crate::x::XWindowID;

/// Dynamically tiled layouts.
pub mod dtiled;
/// Floating layouts.
pub mod floating;

/// The type of layout being used.
#[derive(Clone, Debug, PartialEq)]
pub enum LayoutType {
    /// A simple floating layout style that
    /// does not enforce any rules.
    Floating,
    /// A dynamically tiled layout style that
    /// enforces a master region and satellite windows.
    ///
    /// Similar to XMonad or Qtile.
    DTiled,
    /// User-specified layout.
    Other(String),
}

impl LayoutType {
    /// Construct the `LayoutType::Other` variant.
    pub fn other<S: Into<String>>(name: S) -> LayoutType {
        LayoutType::Other(name.into())
    }

    /// Check whether self is floating.
    ///
    /// Returns false if it is Self::Other(_), even if other is
    /// a floating layout.
    pub fn is_floating(&self) -> bool {
        matches!(self, Self::Floating)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutAction {
    SetMaster(XWindowID),
    UnsetMaster,
    Resize { id: XWindowID, geom: Geometry },
}

// impl LayoutAction {
//     #[inline]
//     pub fn new(id: XWindowID, geom: Geometry) -> Self {
//         Self {
//             id, geom,
//         }
//     }

//     #[inline(always)]
//     pub fn id(&self) -> XWindowID {
//         self.id
//     }

//     #[inline(always)]
//     pub fn geometry(&self) -> Geometry {
//         self.geom
//     }
// }

/// A function that can lay out windows in a user-specified way.
/// Parameters:
/// - &Workspace: the workspace to layout.
/// - &Screen: the screen the workspace is on.
/// - u32: The border width.
/// - f32: The master ratio.
pub type LayoutFn = fn(&Workspace, &Screen, u32, f32) -> Vec<LayoutAction>;

/// An object responsible for arranging layouts within a screen.
/// 
/// Used within a [`WindowManager`] to generate layouts on the fly.
#[derive(Clone)]
pub struct LayoutEngine {
    layout: LayoutType,

    _layoutgen: LayoutFn,
}

impl fmt::Debug for LayoutEngine {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("LayoutEngine")
            .field("layout:", &self.layout)
            .finish()
    }
}

impl LayoutEngine {
    pub fn with_layout(layout: LayoutType, layoutfn: Option<LayoutFn>) -> Self {
        match layout {
            LayoutType::Floating => Self {
                layout,
                _layoutgen: floating::gen_layout,
            },
            LayoutType::DTiled => Self {
                layout,
                _layoutgen: dtiled::gen_layout,
            },
            LayoutType::Other(_) => Self {
                layout,
                _layoutgen: layoutfn.expect("no LayoutFn given"),
            },
        }
    }

    /// Sets the layout being used for the engine.
    /// Does not generate new layouts.
    pub fn set_layout(&mut self, layout: LayoutType, lfn: Option<LayoutFn>) {
        self.layout = layout.clone();
        match layout {
            LayoutType::Floating => self._layoutgen = floating::gen_layout,
            LayoutType::DTiled => self._layoutgen = dtiled::gen_layout,
            LayoutType::Other(_) => self._layoutgen = lfn.expect("no LayoutFn given"),
        }
    }

    /// Returns the current layout being used by the layout engine.
    pub fn layout(&self) -> &LayoutType {
        &self.layout
    }

    /// Generate the layout for the given workspace.
    pub fn gen_layout(&self, ws: &Workspace, scr: &Screen) -> Vec<LayoutAction> {
        //todo: pass in proper border width and ratio numbers
        (self._layoutgen)(ws, scr, crate::types::BORDER_WIDTH, 0.5)
    }
}

/// A trait for implementing layouts.
/// 
/// This will usually be used as a trait object by the manager itself.
pub trait Layout {
    /// The name of the Layout, used to display in some kind of status bar.
    fn name(&self) -> &str;

    /// Generates the actions to be taken to lay out the windows.
    /// Parameters:
    /// - &Workspace: the workspace to layout.
    /// - &Screen: the screen the workspace is on.
    /// - u32: The border width.
    /// - f32: The master ratio.
    fn layout(&mut self, 
        ws: &Workspace, 
        scr: &Screen, 
        bwidth: u32,
        ratio: f32) -> Vec<LayoutAction>;

    /// Returns a boxed version of itself, so it can be used a trait object.
    fn boxed(&self) -> Box<dyn Layout>;
}
