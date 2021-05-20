use crate::core::{Workspace, Screen};
use crate::x::XWindowID;
use crate::types::Geometry;

pub mod floating;
pub mod dtiled;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LayoutType {
    Floating,
    DTiled,
    MTiled,
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

#[derive(Clone, Copy)]
pub struct LayoutEngine {
    layout: LayoutType,

    _layoutgen: fn(&Workspace, &Screen) -> Vec<ResizeAction>,
}

impl LayoutEngine {
    pub fn with_layout(layout: LayoutType) -> Self {
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
        }
    }

    /// Sets the layout being used for the engine.
    /// Does not generate new layouts.
    pub fn set_layout(&mut self, layout: LayoutType) {
        self.layout = layout;
        match layout {
            LayoutType::Floating => {self._layoutgen = floating::gen_layout}
            LayoutType::DTiled => {self._layoutgen = dtiled::gen_layout}
            LayoutType::MTiled => {todo!("Manual tiling not yet implemented")}
        }
    }

    /// Returns the current layout being used by the layout engine.
    pub fn layout(&self) -> LayoutType {
        self.layout
    }
    
    /// Generate the layout for the given workspace.
    pub fn gen_layout(&self, ws: &Workspace, scr: &Screen) -> Vec<ResizeAction> {
        (self._layoutgen)(ws, scr)
    }
}