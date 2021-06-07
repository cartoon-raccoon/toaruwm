use std::fmt;

use crate::core::{Workspace, Screen};
use crate::x::XWindowID;
use crate::types::Geometry;

pub mod floating;
pub mod dtiled;

#[derive(Clone, Debug, PartialEq)]
pub enum LayoutType {
    Floating,
    DTiled,
    MTiled,
    Other(String)
}

#[derive(Clone, Copy, PartialEq)]
pub struct ResizeAction {
    id: XWindowID,
    geom: Geometry,
}

impl ResizeAction {
    #[inline]
    pub fn new(id: XWindowID, geom: Geometry) -> Self {
        Self {
            id, geom,
        }
    }

    #[inline(always)]
    pub fn id(&self) -> XWindowID {
        self.id
    }

    #[inline(always)]
    pub fn geometry(&self) -> Geometry {
        self.geom
    }
}

/// A function that can lay out windows in a user-specified way.
pub type LayoutFn = fn(&Workspace, &Screen) -> Vec<ResizeAction>;

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
                _layoutgen: floating::gen_layout
            },
            LayoutType::DTiled => Self {
                layout,
                _layoutgen: dtiled::gen_layout
            },
            LayoutType::MTiled => {todo!("Manual tiling not yet implemented")}
            LayoutType::Other(_) => Self {
                layout,
                _layoutgen: layoutfn.expect("no LayoutFn given")
            }
        }
    }

    /// Sets the layout being used for the engine.
    /// Does not generate new layouts.
    pub fn set_layout(&mut self, layout: LayoutType, lfn: Option<LayoutFn>) {
        self.layout = layout.clone();
        match layout {
            LayoutType::Floating => {self._layoutgen = floating::gen_layout}
            LayoutType::DTiled => {self._layoutgen = dtiled::gen_layout}
            LayoutType::MTiled => {todo!("Manual tiling not yet implemented")}
            LayoutType::Other(_) => {
                self._layoutgen = lfn.expect("no LayoutFn given")
            }
        }
    }

    /// Returns the current layout being used by the layout engine.
    pub fn layout(&self) -> &LayoutType {
        &self.layout
    }
    
    /// Generate the layout for the given workspace.
    pub fn gen_layout(&self, ws: &Workspace, scr: &Screen) -> Vec<ResizeAction> {
        (self._layoutgen)(ws, scr)
    }
}