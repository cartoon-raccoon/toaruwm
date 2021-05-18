use crate::core::{Workspace, Screen};
use crate::x::XWindowID;
use crate::types::Geometry;

#[derive(Copy, Clone, PartialEq)]
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
            id: id,
            geom: geom,
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
            LayoutType::DTiled => {}
            LayoutType::MTiled => {}
            LayoutType::Floating => {}
        }
        //todo: unimplemented relayout function
        todo!()
    }

    pub fn switch_layout(&mut self, layout: LayoutType) {
        self.layout = layout;
        todo!()
    }

    pub fn current_layout(&self) -> LayoutType {
        self.layout
    }

    pub fn gen_layout(&self, ws: &Workspace, scr: &Screen) -> Vec<ResizeAction> {
        (self._layoutgen)(ws, scr)
    }
}