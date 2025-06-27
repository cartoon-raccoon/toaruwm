use smithay::utils::{
    Rectangle as SmRectangle, 
    Point as SmPoint, 
    Size as SmSize, 
    Logical as SmLogical, 
    Physical as SmPhysical,
    Scale as SmScale,
    Coordinate
};

use crate::types::{Rectangle, Point, Size, Scale, Scalar, Logical, Physical};

impl<N: Scalar> From<SmPoint<N, SmPhysical>> for Point<N, Physical> {
    fn from(smp: SmPoint<N, SmPhysical>) -> Point<N, Physical> {
        let SmPoint {x, y, ..} = smp;

        Point::new(x, y)
    }
}

impl<N: Scalar> From<SmPoint<N, SmLogical>> for Point<N, Logical> {
    fn from(smp: SmPoint<N, SmLogical>) -> Point<N, Logical> {
        let SmPoint {x, y, ..} = smp;

        Point::new(x, y)
    }
}

impl<N: Scalar> From<SmSize<N, SmPhysical>> for Size<N, Physical> {
    fn from(smp: SmSize<N, SmPhysical>) -> Size<N, Physical> {
        let SmSize{w, h,..} = smp;

        Size::new(w, h)
    }
}

impl<N: Scalar> From<SmSize<N, SmLogical>> for Size<N, Logical> {
    fn from(smp: SmSize<N, SmLogical>) -> Size<N, Logical> {
        let SmSize{w, h,..} = smp;

        Size::new(w, h)
    }
}

impl<N: Scalar> From<SmRectangle<N, SmPhysical>> for Rectangle<N, Physical> {
    fn from(smp: SmRectangle<N, SmPhysical>) -> Rectangle<N, Physical> {
        let SmRectangle { loc: SmPoint {x, y, ..}, size: SmSize {w, h, ..}} = smp;

        Rectangle::new(x, y, h, w)
    }
}

impl<N: Scalar> From<SmRectangle<N, SmLogical>> for Rectangle<N, Logical> {
    fn from(smp: SmRectangle<N, SmLogical>) -> Rectangle<N, Logical> {
        let SmRectangle { loc: SmPoint {x, y, ..}, size: SmSize {w, h, ..}} = smp;

        Rectangle::new(x, y, h, w)
    }
}

impl<N: Scalar + Coordinate> From<SmScale<N>> for Scale<N> {
    fn from(smp: SmScale<N>) -> Self {
        Scale {
            x: smp.x,
            y: smp.y,
        }
    }
}