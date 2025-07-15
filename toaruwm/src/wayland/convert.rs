use smithay::utils::{
    Rectangle as SmRectangle, Point as SmPoint, Size as SmSize, Logical as SmLogical, 
    Physical as SmPhysical, Scale as SmScale, Coordinate, Transform as SmTransform,
};
use smithay::output::{Mode, Scale as SmOutputScale};

use crate::types::{Rectangle, Point, Size, Scale, Scalar, Logical, Physical, Transform};
use crate::config::{OutputMode, OutputScale};

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

impl From<SmTransform> for Transform {
    fn from(from: SmTransform) -> Transform {
        match from {
            SmTransform::Normal => Transform::Ident,
            SmTransform::_90    => Transform::Rot90,
            SmTransform::_180   => Transform::Rot180,
            SmTransform::_270   => Transform::Rot270,
            SmTransform::Flipped    => Transform::Flipped,
            SmTransform::Flipped90  => Transform::Flipped90,
            SmTransform::Flipped180 => Transform::Flipped180,
            SmTransform::Flipped270 => Transform::Flipped270,
        }
    }
}

impl From<Mode> for OutputMode {
    fn from(from: Mode) -> OutputMode {
        let Mode {size, refresh} = from;
        OutputMode {
            size: size.into(),
            refresh
        }
    }
}

impl From<SmOutputScale> for OutputScale {
    fn from(from: SmOutputScale) -> OutputScale {
        match from {
            SmOutputScale::Integer(i) => OutputScale::Integer(i),
            SmOutputScale::Fractional(f) => OutputScale::Fractional(f),
            SmOutputScale::Custom{ advertised_integer, fractional } => 
            OutputScale::Split {
                integer: advertised_integer,
                fractional
            }
        }
    }
}