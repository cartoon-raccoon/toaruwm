use crate::Geometry;

pub type XWindowID = u32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XWindow {
    pub id: XWindowID,
    pub geom: Geometry,
}

pub trait XConn {
    fn get_root(&self) -> XWindowID;
    fn get_geometry(&self, window: XWindowID) -> Geometry;
}