//! Primitives for working with geometries.
//! 
//! This module contains the core types [`Scale`], [`Point`], [`Size`], and [`Rectangle`],
//! which are primitives you can use for geometrical operations such as representing window sizes,
//! 
//! All types in this module are generic over a [`Scalar`], which is a type that can act as a scalar
//! in a 2D coordinate space. This trait is implemented for all stable numerical primitive types:
//! 
//! - `u{8,16,32,64,128}`,
//! - `i{8,16,32,64,128}`, and
//! - `f{32,64}`.
//! 
//! ## Physical and Logical Coordinate Spaces
//! 
//! The `Point`, `Size`, and `Rectangle` types are additionally generic over a marker type that implements
//! [`GeometryKind`], which is a marker trait that marks the type as acting in a certain coordinate space.
//! See the [`marker`] module documentation for additional details.
//! 
//! ## Scaling
//! 
//! As the type name suggests, you can use a [`Scale`] to scale another geometrical type. Each of these types
//! implements [`Mul<Scale<N>>`] and [`Div<Scale<N>>`], which upscales and downscales the type respectively.
//! 
//! //todo: example

use core::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, Neg};
use core::cmp::{PartialOrd, Ordering};

use std::marker::PhantomData;

use super::{Cardinal, CardinalX, CardinalY};

pub mod marker {
    //! Marker types for marking Size, Point and Rectangle kind, Physical or Logical.
    //! 
    //! This module contains the [`GeometryKind`] sealed trait,
    //! as well as its two implementors, [`Logical`] and [`Physical`].
    //! These are used to mark whether a `Point` or `Geometry` should be treated
    //! as a physical geometry (i.e. relative to the physical size of a screen),
    //! or as a logical geometry (i.e. relative to other outputs, accounting for their scale).
    //! 
    //! Most applications work with Logical geometries, since that is the most
    //! convenient coordinate space to do operations in among outputs. However, certain things
    //! do require physical coordinates, such as window borders (in order to stay crisp).
    //! 
    //! Most Logical coordinate spaces are linked to a Physical coordinate space by a scale factor.
    //! Conversion methods are provided for you to convert geometrical types between Physical and
    //! Logical spaces, where you provide the scale factor *relative to the Logical space*. That is,
    //! if your Logical space needs to be scaled by a factor of 1.5 to be equal in size to its
    //! corresponding Physical space, when converting from Physical to Logical, *you still
    //! pass in 1.5, **not** 1/1.5*. The conversion method will perform the inversion for you.
    //! 
    //! See the respective `as_logical` and `as_physical` methods for [`Point`][1] and [`Rectangle`][2].
    //! 
    //! [1]: super::Point
    //! [2]: super::Rectangle
    
    mod private {
        pub trait Sealed {}
    }

    use core::ops::{Add, Sub};

    /// A sealed trait for marking types as Scalars, that can be used in Points and Rectangles.
    /// 
    /// A Scalar should have the following properties defined on it:
    /// 
    /// - Multiplication: A Scalar multiplied by another Scalar yields another Scalar.
    /// - Addition: A Scalar added to another Scalar yields another Scalar.
    pub trait Scalar: 
        Sized 
        + Copy 
        + PartialEq 
        + PartialOrd 
        + Add<Self, Output = Self>
        + Sub<Self, Output = Self>
        + Default
    {
        /// The minimum value this scalar can have.
        const MIN: Self;
        
        /// The maximum value this scalar can have.
        const MAX: Self;

        /// The scale identity of the scalar, that is, when scaled by this Scalar, it will have no effect.
        /// Usually, this is the multiplicative identity of the Scalar.
        const SCALE_ID: Self;

        /// The zero point of the Scalar. Usually, this is the additive identity of the Scalar.
        const ZERO: Self;

        /// Convert to this Scalar from an f64.
        fn from_f64(v: f64) -> Self;

        /// Convert this Scalar to an f64.
        fn to_f64(self) -> f64;

        /// Get the absolute value of this Scalar (i.e. distance from its zero).
        fn abs(self) -> Self;

        /// Scale down this Scalar by a scale.
        fn downscale(self, other: Self) -> Self;

        /// Scale up this Scalar by a scale.
        fn upscale(self, other: Self) -> Self;

        #[inline]
        /// Check if the Scalar is greater than its zero.
        fn positive(self) -> bool {
            self > Self::ZERO
        }
    }

    macro_rules! __impl_scalar_signed {
        ($($targ:ty),+) => {
            $(
                impl Scalar for $targ {
                    const MIN: Self = <$targ>::MIN;
                    const MAX: Self = <$targ>::MAX;
                    const SCALE_ID: Self = 1;
                    const ZERO: Self = 0;

                    #[inline]
                    fn from_f64(v: f64) -> Self {
                        v as Self
                    }

                    #[inline]
                    fn to_f64(self) -> f64 {
                        self as f64
                    }

                    #[inline]
                    fn abs(self) -> Self {
                        self.abs()
                    }

                    #[inline]
                    fn downscale(self, other: Self) -> Self {
                        self.saturating_div(other)
                    }

                    #[inline]
                    fn upscale(self, other: Self) -> Self {
                        self.saturating_mul(other)
                    }
                }
            )+
        };
    }

    macro_rules! __impl_scalar_unsigned {
        ($($targ:ty),+) => {
            $(
                impl Scalar for $targ {
                    const MIN: Self = <$targ>::MIN;
                    const MAX: Self = <$targ>::MAX;
                    const SCALE_ID: Self = 1;
                    const ZERO: Self = 0;

                    #[inline]
                    fn from_f64(v: f64) -> Self {
                        v as Self
                    }

                    #[inline]
                    fn to_f64(self) -> f64 {
                        self as f64
                    }

                    #[inline]
                    fn abs(self) -> Self {
                        self
                    }

                    #[inline]
                    fn downscale(self, other: Self) -> Self {
                        self.saturating_div(other)
                    }

                    #[inline]
                    fn upscale(self, other: Self) -> Self {
                        self.saturating_mul(other)
                    }
                }
            )+
        };
    }

    macro_rules! __impl_scalar_floating {
        ($($targ:ty),+) => {
            $(
                impl Scalar for $targ {
                    const MIN: Self = <$targ>::MIN;
                    const MAX: Self = <$targ>::MAX;
                    const SCALE_ID: Self = 1.;
                    const ZERO: Self = 0.;

                    #[inline]
                    fn from_f64(v: f64) -> Self {
                        v as Self
                    }

                    #[inline]
                    fn to_f64(self) -> f64 {
                        self as f64
                    }

                    #[inline]
                    fn abs(self) -> Self {
                        self.abs()
                    }

                    #[inline]
                    fn downscale(self, other: Self) -> Self {
                        self / other
                    }

                    #[inline]
                    fn upscale(self, other: Self) -> Self {
                        self * other 
                    }
                }
            )+
        };
    }

    __impl_scalar_signed!(i8, i16, i32, i64, i128, isize);
    __impl_scalar_unsigned!(u8, u16, u32, u64, u128, usize);
    __impl_scalar_floating!(f32, f64);

    /// A sealed trait defining marker types `Logical` and `Physical`.
    pub trait GeometryKind: Copy + PartialEq + private::Sealed {}

    macro_rules! __impl_geometrykind {
        {$(#[$outer:meta])? $targ:ident} => {
            $(
                #[$outer]
            )?
            #[derive(Debug, Default, Clone, Copy, PartialEq)]
            pub struct $targ;

            impl private::Sealed for $targ {}
            impl GeometryKind for $targ {}
        };
    }

    __impl_geometrykind!{
        /// A type for marking geometrical types as logical.
        Logical
    }
    __impl_geometrykind!{
        /// A type for marking geometrical types as physical.
        Physical
    }


    use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
    /// A marker trait to signal that a type can be treated as a bitmask.
    ///
    /// This means that the type supports bitmask operations such as
    /// bitwise AND, bitwise OR, bitwise NOT, etc.
    pub trait BitMask
    where
        Self: BitAnd + BitOr + Not + BitAndAssign + BitOrAssign + Sized,
    {
    }

    // Blanket implementation for Bitmask
    impl<T> BitMask for T where T: BitAnd + BitOr + Not + BitAndAssign + BitOrAssign + Sized {}
}

pub use marker::{GeometryKind, Logical, Physical, Scalar, BitMask};

/// A two-dimensional Scale that can be used to scale [`Point`]s, [`Size`]s,
/// and [`Rectangle`]s, by multiplying them with it.
/// 
/// A `Scale` tracks its scale factors independently in each dimension, to
/// allow for non-uniform scaling operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Scale<N: Scalar> {
    /// The scale on the X-axis.
    pub x: N,
    /// The scale on the Y-axis.
    pub y: N,
}

impl<N: Scalar> Scale<N> {
    /// Returns a Scale of f64s.
    pub fn to_f64(self) -> Scale<f64> {
        Scale {
            x: self.x.to_f64(),
            y: self.y.to_f64(),
        }
    }

    /// Returns a identity Scale, that has no effect on Scaling operations.
    pub const fn id() -> Scale<N> {
        Scale {
            x: N::SCALE_ID,
            y: N::SCALE_ID,
        }
    }

    /// Returns a uniform Scale, where the `x` and `y` scale factors are equal.
    pub const fn uniform(factor: N) -> Scale<N> {
        Scale {
            x: factor, y: factor
        }
    }
}

impl Scale<f32> {
    /// Returns the reciprocal of `self` (i.e. the reciprocal of each component).
    pub fn recip(self) -> Self {
        Scale {
            x: self.x.recip(),
            y: self.y.recip(),
        }
    }
}

impl Scale<f64> {
    /// Returns the reciprocal of `self` (i.e. the reciprocal of each component).
    pub fn recip(self) -> Self {
        Scale {
            x: self.x.recip(),
            y: self.y.recip(),
        }
    }
}

impl<N: Scalar> Mul for Scale<N> {
    type Output = Self;   

    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.x.upscale(rhs.x),
            y: self.y.upscale(rhs.y),
        }
    }
}

impl<N: Scalar> Div for Scale<N> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Self {
            x: self.x.downscale(rhs.x),
            y: self.y.downscale(rhs.y),
        }
    }
}

impl<N: Scalar> From<(N, N)> for Scale<N> {
    fn from(from: (N, N)) -> Self {
        let (x, y) = from;
        Self {x, y}
    }
}

impl<N: Scalar> From<N> for Scale<N> {
    fn from(from: N) -> Self {
        Self::uniform(from)
    }
}

/// A type for representing a point on a display or screen.
///
/// Implements [`PartialEq`], so you can compare it directly with
/// another Point. You can also directly add and subtract `Point`s,
/// as they implement [`Add`] and [`Sub`] on themselves, and you can
/// also multiply and divide them by [`Scale`]s, as they implement
/// `{Mul,Div}<Scale>`. This simply upscales or downscales them by
/// the given scale.
///
/// # Note
///
/// The (0, 0) reference is by default taken from the top left
/// corner of the 2D plane.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point<N: Scalar, Kind: GeometryKind> {
    /// The `Point`'s X-coordinate.
    pub x: N,
    /// The `Point`'s Y-coordinate.
    pub y: N,
    _kind: PhantomData<Kind>,
}

impl<N, Kind> Add for Point<N, Kind>
where
    N: Scalar,
    Kind: GeometryKind
{
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl<N, Kind> AddAssign for Point<N, Kind>
where
    N: Scalar + AddAssign,
    Kind: GeometryKind {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<N: Scalar, Kind: GeometryKind> Sub for Point<N, Kind> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl<N, Kind> SubAssign for Point<N, Kind>
where
    N: Scalar + SubAssign,
    Kind: GeometryKind
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<N, Kind> Neg for Point<N, Kind>
where
    N: Scalar + Neg<Output = N>,
    Kind: GeometryKind
{
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            _kind: PhantomData,
        }
    }
}

impl<N, Kind> Mul<Scale<N>> for Point<N, Kind>
where
    N: Scalar,
    Kind: GeometryKind
{   
    type Output = Self;

    fn mul(self, rhs: Scale<N>) -> Self {
        self.upscale(rhs)
    }
}

impl<N, Kind> Div<Scale<N>> for Point<N, Kind>
where
    N: Scalar,
    Kind: GeometryKind
{   
    type Output = Self;

    fn div(self, rhs: Scale<N>) -> Self {
        self.downscale(rhs)
    }
}

impl<N, Kind> From<(N, N)> for Point<N, Kind>
where
    N: Scalar,
    Kind: GeometryKind
{
    fn from(from: (N, N)) -> Self {
        let (x, y) = from;
        Self {
            x,
            y,
            _kind: PhantomData, 
        }
    }
}


impl<N: Scalar, Kind: GeometryKind> Point<N, Kind> {
    /// Creates a new Point.
    pub const fn new(x: N, y: N) -> Point<N, Kind> {
        Point { x, y, _kind: PhantomData,}
    }

    /// Creates a new Point where both coordinates are zero.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use toaruwm::types::{Point, Logical};
    /// 
    /// let point1 = Point::<i32, Logical>::zeroed();
    /// let point2 = Point::<i32, Logical>::new(0, 0);
    /// 
    /// assert_eq!(point1, point2);
    /// ```
    pub const fn zeroed() -> Point<N, Kind> {
        Point { x: N::ZERO, y: N::ZERO, _kind: PhantomData}
    }

    /// Converts `self` to a `Point<f64, Kind>`.
    pub fn as_f64(&self) -> Point<f64, Kind> {
        Point {
            x: self.x.to_f64(),
            y: self.y.to_f64(),
            _kind: PhantomData
        }
    }

    /// Calculates the x and y offsets between itself and another Point.
    ///
    /// Offset is calculated with reference to itself.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::{Point, Logical};
    ///
    /// let original = Point::<i32, Logical>::new(50, 50);
    /// let new = Point::<i32, Logical>::new(20, 30);
    ///
    /// let (x, y) = original.calculate_offset(new);
    ///
    /// assert_eq!(x, -30);
    /// assert_eq!(y, -20);
    /// ```
    pub fn calculate_offset(&self, other: Point<N, Kind>) -> (N, N) {
        (other.x - self.x, other.y - self.y)
    }

    /// Calculates the magnitude of the vector formed by this Point, with
    /// the origin (0,0).
    pub fn magnitude(&self) -> N {
        self.distance_to(Point::zeroed())
    }

    /// Calculates the distance to another point, using the
    /// Pythagorean theorem.
    ///
    /// Since most things in this crate take integer values,
    /// you will probably want to round this up/down to
    /// the nearest integer value before coercing to an
    /// integer type.
    pub fn distance_to(&self, other: Point<N, Kind>) -> N {
        let (x, y) = self.calculate_offset(other);

        let ret = ((x.to_f64()).powi(2) + (y.to_f64()).powi(2)).sqrt();

        assert!(!ret.is_nan());
        N::from_f64(ret)
    }

    /// Creates a Point with `delta` in the given direction
    /// (unidirectional offset).
    ///
    /// Only moves the point in one direction.
    // todo: example
    pub fn unidir_offset(&self, delta: N, dir: Cardinal) -> Self {
        use Cardinal::*;

        let Point { x, y, _kind } = *self;

        match dir {
            Up => Point { x, y: y - delta, _kind},
            Down => Point { x, y: y + delta, _kind },
            Left => Point { x: x - delta, y, _kind },
            Right => Point { x: x + delta, y, _kind },
        }
    }

    /// Creates a Point offset by `dx, dy` in the given directions
    /// (bidirectional offset).
    ///
    /// Moves the point in both directions.
    // todo: example
    pub fn bidir_offset(&self, dx: N, dy: N, dirx: CardinalX, diry: CardinalY) -> Self {
        use CardinalX::*;
        use CardinalY::*;

        let Point { x, y, _kind } = *self;

        match (dirx, diry) {
            (Left, Up) => Point {
                x: x - dx,
                y: y - dy,
                _kind
            },
            (Left, Down) => Point {
                x: x - dx,
                y: y + dy,
                _kind
            },
            (Right, Down) => Point {
                x: x + dx,
                y: y + dy,
                _kind
            },
            (Right, Up) => Point {
                x: x + dx,
                y: y - dy,
                _kind
            },
        }
    }

    /// Offsets itself by `delta` in the given direction.
    pub fn unidir_offset_in_place(&mut self, delta: N, dir: Cardinal) {
        let Point { x, y, _kind } = self.unidir_offset(delta, dir);

        self.x = x;
        self.y = y;
    }

    /// Offsets itself by `dx, dy` in the given directions.
    pub fn bidir_offset_in_place(&mut self, dx: N, dy: N, dirx: CardinalX, diry: CardinalY) {
        let Point { x, y, _kind } = self.bidir_offset(dx, dy, dirx, diry);

        self.x = x;
        self.y = y;
    }

    /// Scales up the given Point by a given scale factor on the X and Y axes,
    /// with respect to the origin (0,0) at the top left of the coordinate space.
    pub fn upscale(self, scale: Scale<N>) -> Self {
        let Point {x, y, _kind } = self;

        Self {
            x: x.upscale(scale.x),
            y: y.upscale(scale.y),
            _kind
        }
    }

    /// Scales down the given Point by a given scale factor on the X and Y axes,
    /// with respect to the origin (0,0) at the top left of the coordinate space.
    pub fn downscale(self, scale: Scale<N>) -> Self {
        let Point {x, y, _kind } = self;

        Self {
            x: x.downscale(scale.x),
            y: y.downscale(scale.y),
            _kind
        }
    }
}

impl<N: Scalar> Point<N, Logical> {
    /// Returns a `Point<Physical>`, scaled by `scale`.
    pub fn as_physical(self, scale: Scale<N>) -> Point<N, Physical> {
        let Point {x, y, ..} = self.upscale(scale);

        Point {
            x, y, _kind: PhantomData
        }
    }
}

impl<N: Scalar> Point<N, Physical> {
    /// Returns a `Point<Logical>`, scaled by **`1 / scale`.**
    pub fn as_logical(self, scale: Scale<N>) -> Point<N, Logical> {
        let Point {x, y, ..} = self.downscale(scale);

        Point {
            x, y, _kind: PhantomData
        }
    }
}

/// A type for representing a 2D rectangular space, without
/// respect to its position on the coordinate space.
/// 
/// Implements [`PartialEq`], so you can compare it directly
/// with another Size.
/// 
/// [`PartialOrd`] is implemented with respect to area, and
/// is only implemented if the `Scalar` generic implements [`Mul`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size<N: Scalar, Kind: GeometryKind> {
    /// The width of the Size.
    pub width: N,
    /// The height of the Size.
    pub height: N,

    _kind: PhantomData<Kind>,
}

impl<N, Kind> PartialOrd for Size<N, Kind>
where
    N: Scalar + Mul<Output=N>,
    Kind: GeometryKind
{
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.area().partial_cmp(&rhs.area())
    }
}

impl<N, Kind> Mul<Scale<N>> for Size<N, Kind>
where
    N: Scalar,
    Kind: GeometryKind
{
    type Output = Self;

    fn mul(self, rhs: Scale<N>) -> Self {
        self.upscale(rhs)
    }
}

impl<N, Kind> Div<Scale<N>> for Size<N, Kind>
where
    N: Scalar,
    Kind: GeometryKind
{
    type Output = Self;

    fn div(self, rhs: Scale<N>) -> Self {
        self.downscale(rhs)
    }
}

impl<N, Kind> From<(N, N)> for Size<N, Kind>
where
    N: Scalar,
    Kind: GeometryKind
{
    fn from(from: (N, N)) -> Self {
        let (width, height) = from;

        Self {
            width,
            height,
             _kind: PhantomData,
        }
    }
}

impl<N: Scalar, Kind: GeometryKind> Size<N, Kind> {
    /// Creates a new Size.
    pub const fn new(width: N, height: N) -> Self {
        Self {
            width,
            height,
            _kind: PhantomData
        }
    }

    /// Creates a new Size with all fields set to zero.
    pub const fn zeroed() -> Self {
        Self::new(N::ZERO, N::ZERO)
    }

    /// Converts `self` to a `Point<f64, Kind>`.
    pub fn as_f64(&self) -> Size<f64, Kind> {
        Size {
            width: self.width.to_f64(),
            height: self.height.to_f64(),
            _kind: PhantomData,
        }
    }

    /// Returns true if the area of this size is zero.
    pub fn is_empty(&self) -> bool {
        self.width == N::ZERO || self.height == N::ZERO
    }

    /// Upscales the given Point by a given scale factor on the X and Y axes,
    /// with respect to the origin (0,0) at the top left of the coordinate space.
    pub fn upscale(self, scale: Scale<N>) -> Self {
        let Size {width, height, _kind} = self;

        Size {
            width: width.upscale(scale.x),
            height: height.upscale(scale.y),
            _kind
        }
    }

    /// Downscales the given Point by a given scale factor on the X and Y axes,
    /// with respect to the origin (0,0) at the top left of the coordinate space.
    pub fn downscale(self, scale: Scale<N>) -> Self {
        let Size {width, height, _kind} = self;

        Size {
            width: width.downscale(scale.x),
            height: height.downscale(scale.y),
            _kind
        }
    }
}

impl<N: Scalar + Mul<Output=N>, Kind: GeometryKind> Size<N, Kind> {
    /// Returns the area of the size (width * height).
    pub fn area(&self) -> N {
        self.width * self.height
    }
}

impl<N: Scalar> Size<N, Logical> {
    /// Upscales a Size by the given scale, returning it as a Physical size.
    pub fn as_physical(&self, scale: Scale<N>) -> Size<N, Physical> {
        let Size {width, height, .. } = self.upscale(scale);

        Size {
            width,
            height,
            _kind: PhantomData,
        }
    }
}

impl<N: Scalar> Size<N, Physical> {
    /// Downscales a Size by the given Scale, returning it as a Logical size.
    pub fn as_logical(&self, scale: Scale<N>) -> Size<N, Logical> {
        let Size {width, height, .. } = self.downscale(scale);

        Size {
            width,
            height,
            _kind: PhantomData,
        }
    }
}


/// A type for representing a 2D rectangular space, anchored to a
/// Point on the coordinate space.
///
/// Implements [`PartialEq`], so you can compare it directly with
/// another Rectangle.
///
/// # Note on Gravity
///
/// Rectangles follow the default of taking their gravity from the top-left 
/// corner, that is, (0, 0) is considered the top left corner
/// of the screen, and any increase is an offset to the right
/// or downwards.
///
/// _Note:_ The Default impl returns Rectangle {0, 0, 0, 0}.
/// 
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle<N: Scalar, Kind: GeometryKind> {
    /// The point that the Rectangle is anchored to.
    pub point: Point<N, Kind>,
    /// The size of the Rectangle.
    pub size: Size<N, Kind>,
}

impl<N: Scalar, Kind: GeometryKind> Default for Rectangle<N, Kind> {
    fn default() -> Self {
        Rectangle::zeroed()
    }
}

impl<N: Scalar, Kind: GeometryKind> Rectangle<N, Kind> {
    /// Constructs a new `Rectangle`.
    pub fn new(x: N, y: N, h: N, w: N) -> Self {
        Rectangle {
            point: Point::new(x, y),
            size: Size::new(w, h)
        }
    }

    /// Creates a new `Rectangle` of size (0, 0), anchored at the given `point`.
    pub fn from_point(point: Point<N, Kind>) -> Self {
        Rectangle {
            point,
            size: Size::zeroed()
        }
    }

    /// Creates a new Rectangle of the given `size`, anchored at the origin (0, 0)
    pub fn from_size(size: Size<N, Kind>) -> Self {
        Rectangle {
            point: Point::zeroed(),
            size
        }
    }

    /// Convenience function for constructing a `Rectangle` with all fields
    /// set to zero.
    pub fn zeroed() -> Self {
        Rectangle::new(N::ZERO, N::ZERO, N::ZERO, N::ZERO)
    }

    /// Creates a `Rectangle` based at the origin (0, 0)
    /// with the given dimensions `height` and `width`.
    pub fn at_origin(height: N, width: N) -> Self {
        Self::new(N::ZERO, N::ZERO, height, width)
    }

    /// Converts `self` to a `Point<f64, Kind>`.
    pub fn as_f64(&self) -> Rectangle<f64, Kind> {
        Rectangle {
            point: self.point.as_f64(),
            size: self.size.as_f64(),
        }
    }

    /// Converts `self` to a `Point<f64, Kind>`.
    pub fn is_empty(&self) -> bool {
        self.size.is_empty()
    }

    /// Check whether this Rectangle encloses another Rectangle.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::{Rectangle, Logical};
    ///
    /// let original = Rectangle::<i32, Logical>::new(0, 0, 100, 200);
    ///
    /// let new = Rectangle::<i32, Logical>::new(2, 2, 50, 75);
    ///
    /// assert!(original.contains(&new));
    /// ```
    pub fn contains(&self, other: &Self) -> bool {
        match other {
            Rectangle { 
                point: Point { x, .. }, 
                .. 
            } if *x < self.point.x => false,
            Rectangle { 
                point: Point { x, .. }, 
                size: Size {width, .. }, 
                ..
            } if (*x + *width) > (self.point.x + self.size.width) => false,
            Rectangle { 
                point: Point { y, .. },
                .. 
            } if *y < self.point.y => false,
            Rectangle { 
                point: Point { y, .. }, 
                size: Size { height, .. },
                ..
            } if (*y + *height) > (self.point.y + self.size.height) => false,
            _ => true,
        }
    }

    /// Check whether this Rectangle contains a certain point.
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::{Rectangle, Point, Logical};
    ///
    /// let original = Rectangle::<i32, Logical>::new(0, 0, 100, 200);
    ///
    /// let point = Point::<i32, Logical>::new(50, 50);
    ///
    /// assert!(original.contains_point(point));
    /// ```
    pub fn contains_point(&self, pt: Point<N, Kind>) -> bool {
        let wrange = self.point.x..(self.point.x + self.size.width);
        let hrange = self.point.y..(self.point.y + self.size.height);

        wrange.contains(&pt.x) && hrange.contains(&pt.y)
    }

    /// Check whether this Rectangle overlaps with `other`.
    // todo: doctest and example
    pub fn overlaps_with(&self, other: Self) -> bool {
        let a_left = self.point.x;
        let a_right = self.point.x + self.size.width;
        let a_top = self.point.y;
        let a_bot = self.point.y + self.size.height;

        let b_left = other.point.x;
        let b_right = other.point.x  + other.size.width;
        let b_top = other.point.y;
        let b_bot = other.point.y + other.size.height;

        // the complement of the logical OR of the four conditions,
        // any one of which would guarantee that there is no overlap
        !(
            // 1. my left edge is to the right of their right
            a_left > b_right ||
            // 2. my right edge is to the left of their left
            a_right < b_left ||
            // 3. my top edge is below their bottom
            a_top > b_bot ||
            // 4. my bottom edge is above their top
            a_bot < b_top
        )
    }

    /// Trim off an area from a `Rectangle` from the side corresponding
    /// to `dir` (`Cardinal::Up` trims the top, `CardinaL::Down`
    /// trims the bottom).
    ///
    /// This returns a new Geometry.
    #[must_use]
    pub fn trim(&self, trim: N, dir: Cardinal) -> Self {
        use Cardinal::*;
        match dir {
            Up => Rectangle::new(self.point.x, self.point.y + trim, self.size.height - trim, self.size.width),
            Down => Rectangle::new(self.point.x, self.point.y, self.size.height - trim, self.size.width),
            Left => Rectangle::new(self.point.x + trim, self.point.y, self.size.height, self.size.width - trim),
            Right => Rectangle::new(self.point.x, self.point.y, self.size.height, self.size.width - trim),
        }
    }

    /// Creates a new `Rectangle` offset by `delta` pixels in the given
    /// direction `dir` (unidirectional offset).
    pub fn unidir_offset(&self, delta: N, dir: Cardinal) -> Self {
        let Rectangle {point, size} = *self;

        let point = point.unidir_offset(delta, dir);

        Rectangle {point, size}
    }

    /// Creates a new `Rectangle` offset by `dx, dy` pixels in the given
    /// directions `dirx, diry` (bidirectional offset).
    pub fn bidir_offset(&self, dx: N, dy: N, dirx: CardinalX, diry: CardinalY) -> Self {
        let Rectangle {point, size} = *self;

        let point = point.bidir_offset(dx, dy, dirx, diry);

        Rectangle {point, size}
    }

    /// Returns a Rectangle formed by the intersection of another Geometry.
    /// This is effectively a set containing all points found in both Geometries.
    pub fn intersect(&self, _other: Rectangle<N, Kind>) -> Self {
        todo!()
    }

    /// Returns a Rectangle upscaled by a given Scale.
    /// Also scales the Rectangle's position with respect to the origin (0, 0).
    pub fn upscale(self, scale: Scale<N>) -> Self {
        Self {
            point: self.point.upscale(scale),
            size: self.size.upscale(scale),
        }
    }

    /// Returns a Rectangle downscaled by a given Scale.
    pub fn downscale(self, scale: Scale<N>) -> Self {
        Self {
            point: self.point.downscale(scale),
            size: self.size.downscale(scale),
        }
    }
}

impl<N: Scalar> Rectangle<N, Logical> {
    /// Returns a `Rectangle<Physical>`, scaled by `scale`.
    pub fn as_physical(self, scale: Scale<N>) -> Rectangle<N, Physical> {
        let point = self.point.as_physical(scale);
        let size = self.size.as_physical(scale);

        Rectangle {point, size}
    }
}

impl<N: Scalar> Rectangle<N, Physical> {
    /// Returns a `Rectangle<Logical>`, scaled by **`1 / scale`.**
    pub fn as_logical(self, scale: Scale<N>) -> Rectangle<N, Logical> {
        let point = self.point.as_logical(scale);
        let size = self.size.as_logical(scale);

        Rectangle {point, size}
    }
}

impl<Kind: GeometryKind> Rectangle<i32, Kind> {
    /// Splits a Rectangle into `n` parts horizontally, each part
    /// covering a region of the original Geometry, top down.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::{Rectangle, Logical};
    ///
    /// let original = Rectangle::<i32, Logical>::new(0, 0, 100, 200);
    ///
    /// let new_geoms = original.split_horz_n(2);
    ///
    /// assert_eq!(new_geoms, vec![
    ///     Rectangle::<i32, Logical>::new(0, 0, 50, 200),
    ///     Rectangle::<i32, Logical>::new(0, 50, 50, 200),
    /// ]);
    /// ```
    #[must_use]
    pub fn split_horz_n(&self, n: i32) -> Vec<Self> {
        let new_height = self.size.height / n as i32;

        let mut ret = Vec::new();

        let _kind = PhantomData;

        for i in 0..n {
            ret.push(Rectangle {
                point: Point { 
                    x: self.point.x,
                    y: self.point.y + (i as i32 * new_height),
                    _kind,
                },
                size: Size {
                    width: self.size.width,
                    height: new_height,
                    _kind
                }
            })
        }

        ret
    }

    /// Splits a Rectangle into `n` parts vertically, each part
    /// covering a region of the original Geometry, from left.
    ///
    /// Does *not* split in place.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Geometry;
    ///
    /// let original = Geometry::new(0, 0, 100, 200);
    ///
    /// let new_geoms = original.split_vert_n(2);
    ///
    /// assert_eq!(new_geoms, vec![
    ///     Geometry::new(0, 0, 100, 100),
    ///     Geometry::new(100, 0, 100, 100),
    /// ]);
    /// ```
    #[must_use]
    pub fn split_vert_n(&self, n: usize) -> Vec<Self> {
        let new_width = self.size.width / n as i32;

        let mut ret = Vec::new();

        let _kind = PhantomData;

        for i in 0..n {
            ret.push(Rectangle {
                point: Point {
                    x: self.point.x + (i as i32 * new_width),
                    y: self.point.y,
                    _kind
                },
                size: Size {
                    width: new_width,
                    height: self.size.height,
                    _kind
                }
            })
        }

        ret
    }

    /// Splits a Geometry into two parts horizontally by a given ratio
    /// where the ratio is the fraction of the original height.
    /// The ratio is clamped between 0.0 and 1.0.
    ///
    /// Returns `(top, bottom)`.
    ///
    /// Works best with clean ratios such as 0.5, 0.75, 0.6 etc.
    ///
    /// # Panics
    ///
    /// Panics if ratio is `f32::NAN`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Geometry;
    ///
    /// let original = Geometry::new(0, 0, 100, 200);
    ///
    /// let (top, bottom) = original.split_horz_ratio(0.75);
    ///
    /// assert_eq!(top, Geometry::new(0, 0, 75, 200));
    /// assert_eq!(bottom, Geometry::new(0, 75, 25, 200));
    /// ```
    #[must_use]
    pub fn split_horz_ratio(&self, ratio: f32) -> (Self, Self) {
        let ratio = ratio.clamp(0.0, 1.0);

        if ratio.is_nan() {
            panic!("Got f32::NAN");
        }

        let top_height = (self.size.height as f32 * ratio) as i32;
        let bottom_height = self.size.height - top_height;

        let _kind = PhantomData;

        (
            // top
            Rectangle {
                point: Point {
                    x: self.point.x,
                    y: self.point.y,
                    _kind
                },
                size: Size {
                    width: self.size.width,
                    height: top_height,
                    _kind
                }
            },
            // bottom
            Rectangle {
                point: Point {
                    x: self.point.x,
                    y: self.point.y + top_height,
                    _kind,
                },
                size: Size {
                    height: bottom_height,
                    width: self.size.width,
                    _kind
                }
            },
        )
    }

    /// Splits a Geometry into two parts vertically by a given ratio
    /// where the ratio is the fraction of the original width.
    /// The ratio is clamped between 0.0 and 1.0.
    ///
    /// Returns `(left, right)`.
    ///
    /// Works best with clean ratios such as 0.5, 0.75, 0.6 etc.
    ///
    /// # Panics
    ///
    /// Panics if ratio is `f32::NAN`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Geometry;
    ///
    /// let original = Geometry::new(0, 0, 100, 200);
    ///
    /// let (top, bottom) = original.split_vert_ratio(0.75);
    ///
    /// assert_eq!(top, Geometry::new(0, 0, 100, 150));
    /// assert_eq!(bottom, Geometry::new(150, 0, 100, 50));
    /// ```
    #[must_use]
    pub fn split_vert_ratio(&self, ratio: f32) -> (Self, Self) {
        let ratio = ratio.clamp(0.0, 1.0);

        if ratio.is_nan() {
            panic!("Got f32::NAN");
        }

        let left_width = (self.size.width as f32 * ratio) as i32;
        let right_width = self.size.width - left_width;

        let _kind = PhantomData;

        (
            // left
            Rectangle {
                point: Point {
                    x: self.point.x,
                    y: self.point.y,
                    _kind,
                },
                size: Size {
                    width: left_width,
                    height: self.size.height,
                    _kind,
                }
            },
            // right
            Rectangle {
                point: Point {
                    x: self.point.x + left_width,
                    y: self.point.y,
                    _kind: PhantomData,
                },
                size: Size {
                    width: right_width,
                    height: self.size.height,
                    _kind: PhantomData,
                }
            },
        )
    }

    /// Splits a Geometry _horizontally_ into two Geometries
    /// at a given height.
    ///
    /// Returns (top, bottom), where bottom has the given height.
    #[must_use]
    pub fn split_at_height(&self, height: i32) -> (Self, Self) {
        (
            // Top
            Rectangle {
                point: Point {
                    x: self.point.x,
                    y: self.point.y,
                    _kind: PhantomData,
                },
                size: Size {
                    width: self.size.width,
                    height: self.size.height - height,
                    _kind: PhantomData,
                }
            },
            // Bottom
            Rectangle {
                point: Point {
                    x: self.point.x,
                    y: self.point.y + height,
                    _kind: PhantomData,
                },
                size: Size {
                    height,
                    width: self.size.width,
                    _kind: PhantomData,
                }
            },
        )
    }

    /// Splits a Geometry _horizontally_ into two Geometries
    /// at a given width.
    ///
    /// Returns (left, right), where left has the given width.
    #[must_use]
    pub fn split_at_width(&self, width: i32) -> (Self, Self) {
        (
            // Left
            Rectangle {
                point: Point {
                    x: self.point.x,
                    y: self.point.y,
                    _kind: PhantomData,
                },
                size: Size {
                    height: self.size.height,
                    width,
                    _kind: PhantomData,
                }
            },
            // Right
            Rectangle {
                point: Point {
                    x: self.point.x + width,
                    y: self.point.y,
                    _kind: PhantomData,
                },
                size: Size {
                    width: self.size.width - width,
                    height: self.size.height,
                    _kind: PhantomData,
                }
            },
        )
    }
}

impl<N, Kind> Mul<Scale<N>> for Rectangle<N, Kind>
where
    N: Scalar,
    Kind: GeometryKind
{
    type Output = Self;

    fn mul(self, scale: Scale<N>) -> Self {
        Self {
            point: self.point.upscale(scale),
            size: self.size.upscale(scale)
        }
    }
}

impl<N, Kind> Div<Scale<N>> for Rectangle<N, Kind>
where
    N: Scalar,
    Kind: GeometryKind
{
    type Output = Self;

    fn div(self, scale: Scale<N>) -> Self {
        Self {
            point: self.point.downscale(scale),
            size: self.size.downscale(scale)
        }
    }
}