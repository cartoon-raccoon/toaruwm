use smithay::utils::{
    Rectangle as SmRectangle, 
    Point as SmPoint, 
    Size as SmSize, 
    Logical as SmLogical, 
    Physical as SmPhysical};

use crate::types::{Rectangle, Point, Logical, Physical};

impl<N: Into<i32>> From<SmPoint<N, SmPhysical>> for Point<Physical> {
    fn from(smp: SmPoint<N, SmPhysical>) -> Point<Physical> {
        let SmPoint {x, y, ..} = smp;

        Point::new(x, y)
    }
}

impl<N: Into<i32>> From<SmPoint<N, SmLogical>> for Point<Logical> {
    fn from(smp: SmPoint<N, SmLogical>) -> Point<Logical> {
        let SmPoint {x, y, ..} = smp;

        Point::new(x, y)
    }
}

impl<N: Into<i32>> From<SmRectangle<N, SmPhysical>> for Rectangle<Physical> {
    fn from(smp: SmRectangle<N, SmPhysical>) -> Rectangle<Physical> {
        let SmRectangle { loc: SmPoint {x, y, ..}, size: SmSize {w, h, ..}} = smp;

        Rectangle::new(x, y, h, w)
    }
}

impl<N: Into<i32>> From<SmRectangle<N, SmLogical>> for Rectangle<Logical> {
    fn from(smp: SmRectangle<N, SmLogical>) -> Rectangle<Logical> {
        let SmRectangle { loc: SmPoint {x, y, ..}, size: SmSize {w, h, ..}} = smp;

        Rectangle::new(x, y, h, w)
    }
}