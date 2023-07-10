use std::fmt;

use crate::core::{Screen, Workspace};
use crate::types::Geometry;
use crate::x::XWindowID;

pub mod dtiled;
pub mod floating;

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
    /// A manually tiled layout style that
    /// enforces equal-sized windows.
    MTiled,
    /// User-specified layout.
    Other(String),
}

impl LayoutType {
    pub fn other(name: &str) -> LayoutType {
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

#[derive(Clone, Copy, PartialEq)]
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
            LayoutType::MTiled => {
                todo!("Manual tiling not yet implemented")
            }
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
            LayoutType::MTiled => {
                todo!("Manual tiling not yet implemented")
            }
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
