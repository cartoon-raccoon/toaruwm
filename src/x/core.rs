use crate::Geometry;
use crate::XWinProperties;

pub type XWindowID = u32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XWindow {
    pub id: XWindowID,
    pub geom: Geometry,
}

pub trait XConn {
    fn get_root(&self) -> XWindowID;
    fn get_geometry(&self, window: XWindowID) -> Geometry;
    fn get_client_properties(&self, window: XWindowID) -> XWinProperties;
}