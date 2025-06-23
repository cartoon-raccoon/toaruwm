//! Basic core types used throughout this crate at a high level.
//!
//! # Important Note
//!
//! For types pertaining to the cartesian plane,
//! all of them assume the default X server window gravity (NorthWest).
//!
//! For X server-specific types, see [`crate::x::core`].

use core::ops::{Add, Sub, AddAssign, SubAssign, Neg};
use core::cmp::{PartialOrd, Ordering};


#[doc(inline)]
pub use crate::core::{Ring, Selector};

use std::hash::Hash;
use std::fmt::Debug;
use std::collections::HashMap;
use std::any::Any;
use std::marker::PhantomData;

/// A type that can uniquely identify any client connected to a
/// running ToaruWM instance.
/// 
/// It is backend-agnostic, and each backend provides their own
/// type that implements this trait.
pub trait ClientId: Debug + Clone + Eq + Hash {}

/// Data about a given client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientData {
    geom: Rectangle<Logical>,
    urgent: bool,
}

/// A general dictionary type that can store most variable-value mappings.
pub type Dict = HashMap<String, Box<dyn Any>>;

/// Macro for quick-creating a new Dict.
/// 
/// Note: The value you insert should not be boxed, as this macro
/// creates a new Box around `$val`.
#[macro_export]
macro_rules! dict {
    {} => {Dict::new()};
    {$($key:literal, $val:expr);+} => {
        {
            let mut __dict = Dict::new();
    
            $(
                __dict.insert(String::from($key), Box::new($val));
            )+
    
            __dict
        }
    };
}

/// Specifies a direction.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

/// A cardinal direction.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cardinal {
    Up,
    Down,
    Left,
    Right,
}

/// A subset of Cardinal with only Left and Right variants.
///
/// It is disjoint with `CardinalY`.
/// _Note:_ This type can be converted from a standard [`Cardinal`],
/// but the conversion is lossy: `Cardinal::Down` will be converted
/// to a `CardinalX::Left`, and `Cardinal::Up` to a `CardinalX::Right`.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardinalX {
    Left,
    Right,
}

impl From<CardinalX> for Cardinal {
    fn from(from: CardinalX) -> Cardinal {
        match from {
            CardinalX::Left  => Cardinal::Left,
            CardinalX::Right => Cardinal::Right,
        }
    }
}

impl From<Cardinal> for CardinalX {
    fn from(from: Cardinal) -> CardinalX {
        match from {
            Cardinal::Left | Cardinal::Down => CardinalX::Left,
            Cardinal::Right | Cardinal::Up => CardinalX::Right,
        }
    }
}

/// A subset of Cardinal with only Up and Down variants.
///
/// It is disjoint with `CardinalX`.
/// 
/// _Note:_ This type can be converted from a standard [`Cardinal`],
/// but the conversion is lossy: `Cardinal::Left` will be converted
/// to a `CardinalY::Down`, and `Cardinal::Right` to a `CardinalY::Up`.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardinalY {
    Up,
    Down,
}

impl From<CardinalY> for Cardinal {
    fn from(from: CardinalY) -> Cardinal {
        match from {
            CardinalY::Up  => Cardinal::Up,
            CardinalY::Down => Cardinal::Down,
        }
    }
}

impl From<Cardinal> for CardinalY {
    fn from(from: Cardinal) -> CardinalY {
        match from {
            Cardinal::Left | Cardinal::Down => CardinalY::Down,
            Cardinal::Right | Cardinal::Up => CardinalY::Up,
        }
    }
}

/// The direction in which `Point`s and `Geometry`s
/// should take reference to when calculating offsets.
///
/// For example, if a Geometry takes a Gravity of `NorthWest`,
/// then when the Geometry is resized, it will resize
/// towards the top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gravity {
    /// The top left direction.
    NorthWest,
    /// The top direction.
    North,
    /// The top right direction.
    NorthEast,
    /// The left direction.
    West,
    /// Dead center in the Rectangle.
    Center,
    /// The right direction.
    East,
    /// The bottom left direction.
    SouthWest,
    /// The bottom direction.
    South,
    /// The bottom right direction.
    SouthEast,
}

pub mod marker {
    //! Marker types for marking Point and Rectangle kind, Physical or Logical.
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

    /// A trait defining marker types `Logical` and `Physical`.
    pub trait GeometryKind: Copy + PartialEq + private::Sealed {}

    /// A marker type indicating a geometry type is logical.
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub struct Logical;

    impl GeometryKind for Logical {}
    impl private::Sealed for Logical {}

    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    /// A marker type indicating a geometry type is physical.
    pub struct Physical;

    impl GeometryKind for Physical {}
    impl private::Sealed for Physical {}


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

pub use marker::{GeometryKind, Logical, Physical, BitMask};

/// A type for representing a point on a display or screen.
///
/// Implements [`PartialEq`][1], so you can compare it directly with
/// another Point.
///
/// # Note
///
/// The (0, 0) reference is by default taken from the top left
/// corner of the 2D plane.
///
/// [1]: std::cmp::PartialEq
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point<Kind: GeometryKind> {
    pub x: i32,
    pub y: i32,
    _kind: PhantomData<Kind>,
}

impl<Kind: GeometryKind> Add for Point<Kind> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl<Kind: GeometryKind> AddAssign for Point<Kind> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<Kind: GeometryKind> Sub for Point<Kind> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl<Kind: GeometryKind> SubAssign for Point<Kind> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<Kind: GeometryKind> Neg for Point<Kind> {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}


impl<Kind: GeometryKind> Point<Kind> {
    /// Creates a new Point.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Point;
    ///
    /// let point = Point::new(0, 0);
    ///
    /// assert_eq!(point, Point {x: 0, y: 0});
    /// ```
    pub fn new<N: Into<i32>>(x: N, y: N) -> Point<Kind> {
        Point { x: x.into(), y: y.into(), _kind: PhantomData,}
    }

    /// Creates a new Point where both coordinates are zero.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use toaruwm::types::Point;
    /// 
    /// let point = Point::zeroed();
    /// 
    /// assert_eq!(point, Point {x: 0, y: 0});
    /// ```
    pub const fn zeroed() -> Point<Kind> {
        Point { x: 0, y: 0, _kind: PhantomData}
    }

    /// Calculates the x and y offsets between itself and another Point.
    ///
    /// Offset is calculated with reference to itself.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Point;
    ///
    /// let original = Point::new(50, 50);
    /// let new = Point::new(20, 30);
    ///
    /// let (x, y) = original.calculate_offset(new);
    ///
    /// assert_eq!(x, -30);
    /// assert_eq!(y, -20);
    /// ```
    pub fn calculate_offset(&self, other: Point<Kind>) -> (i32, i32) {
        (other.x - self.x, other.y - self.y)
    }

    /// Calculates the magnitude of the vector formed by this Point, with
    /// the origin (0,0).
    pub fn magnitude(&self) -> f32 {
        self.distance_to(Point::zeroed())
    }

    /// Calculates the distance to another point, using the
    /// Pythagorean theorem.
    ///
    /// Since most things in this crate take integer values,
    /// you will probably want to round this up/down to
    /// the nearest integer value before coercing to an
    /// integer type.
    pub fn distance_to(&self, other: Point<Kind>) -> f32 {
        let (x, y) = self.calculate_offset(other);

        let ret = ((x as f32).powi(2) + (y as f32).powi(2)).sqrt();

        assert!(!ret.is_nan());
        ret
    }

    /// Creates a Point with `delta` in the given direction
    /// (unidirectional offset).
    ///
    /// Only moves the point in one direction.
    // todo: example
    pub fn unidir_offset(&self, delta: i32, dir: Cardinal) -> Self {
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
    pub fn bidir_offset(&self, dx: i32, dy: i32, dirx: CardinalX, diry: CardinalY) -> Self {
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
    pub fn unidir_offset_in_place(&mut self, delta: i32, dir: Cardinal) {
        let Point { x, y, _kind } = self.unidir_offset(delta, dir);

        self.x = x;
        self.y = y;
    }

    /// Offsets itself by `dx, dy` in the given directions.
    pub fn bidir_offset_in_place(&mut self, dx: i32, dy: i32, dirx: CardinalX, diry: CardinalY) {
        let Point { x, y, _kind } = self.bidir_offset(dx, dy, dirx, diry);

        self.x = x;
        self.y = y;
    }

    /// Scales the given Point by a given scale factor on the X and Y axes,
    /// with respect to the origin (0,0) at the top left of the coordinate space.
    pub fn scale(&self, scale_x: f32, scale_y: f32) -> Self {
        self.scale_gen::<Kind>(scale_x, scale_y)
    }

    pub(crate) fn scale_gen<K: GeometryKind>(&self, scale_x: f32, scale_y: f32) -> Point<K> {
        let Point { x, y, .. } = *self;

        Point {
            x: ((x as f32) * scale_x).round() as i32,
            y: ((y as f32) * scale_y).round() as i32,
            _kind: PhantomData,
        }
    }
}

impl Point<Logical> {
    /// Returns a `Point<Physical>`, scaled by `scale`.
    pub fn as_physical(&self, scale: f32) -> Point<Physical> {
        self.scale_gen::<Physical>(scale, scale)
    }
}

impl Point<Physical> {
    /// Returns a `Point<Logical>`, scaled by **`1 / scale`.**
    pub fn as_logical(&self, scale: f32) -> Point<Logical> {
        // account for if scale == 0, since calling recip on 0 will give
        // a divide-by-zero error
        let inverse = if scale == 0. {0.} else {scale.recip()};
        self.scale_gen::<Logical>(inverse, inverse)
    }
}

/// A type for representing a 2D rectangular space, without
/// respect to its position on the coordinate space.
/// 
/// Implements [`PartialEq`][1], so you can compare it directly
/// with another Size.
/// 
/// [1]: std::cmp::PartialEq
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size<Kind: GeometryKind> {
    /// The width of the Size.
    pub width: i32,
    /// The height of the Size.
    pub height: i32,

    _kind: PhantomData<Kind>,
}

impl<Kind: GeometryKind> PartialOrd for Size<Kind> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.area().partial_cmp(&rhs.area())
    }
}

impl<Kind: GeometryKind> Size<Kind> {
    /// Creates a new Size.
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            _kind: PhantomData
        }
    }

    /// Creates a new Size with all fields set to zero.
    pub fn zeroed() -> Self {
        Self::new(0, 0)
    }

    /// Returns the area of the size (width * height).
    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    /// Scales the given Point by a given scale factor on the X and Y axes,
    /// with respect to the origin (0,0) at the top left of the coordinate space.
    pub fn scale(&self, scale_x: f32, scale_y: f32) -> Self {
        self.scale_gen::<Kind>(scale_x, scale_y)
    }

    pub(crate) fn scale_gen<K: GeometryKind>(&self, scale_x: f32, scale_y: f32) -> Size<K> {
        let Size {width, height, ..} = *self;

        Size {
            width: ((width as f32) * scale_x).round() as i32,
            height: ((height as f32) * scale_y).round() as i32,
            _kind: PhantomData,
        }
    }
}

impl Size<Logical> {
    /// Returns a `Size<Physical>`, scaled by `scale`.
    pub fn as_physical(&self, scale: f32) -> Size<Physical> {
        self.scale_gen::<Physical>(scale, scale)
    }
}

impl Size<Physical> {
    /// Returns a `Size<Logical>`, scaled by **`1 / scale`.**
    pub fn as_logical(&self, scale: f32) -> Size<Logical> {
        let inverse = if scale == 0. {0.} else {scale.recip()};
        self.scale_gen::<Logical>(inverse, inverse)
    }
}


/// A type for representing a 2D rectangular space, anchored to a
/// Point on the coordinate space.
///
/// Implements [`PartialEq`][1], so you can compare it directly with
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
/// [1]: std::cmp::PartialEq
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle<Kind: GeometryKind> {
    /// The point that the Rectangle is anchored to.
    pub point: Point<Kind>,
    /// The size of the Rectangle.
    pub size: Size<Kind>,
}

impl<Kind: GeometryKind> Default for Rectangle<Kind> {
    fn default() -> Self {
        Rectangle::new(0, 0, 0, 0)
    }
}

impl<Kind: GeometryKind> Rectangle<Kind> {
    /// Constructs a new Geometry.
    pub fn new<N: Into<i32>>(x: N, y: N, h: N, w: N) -> Self {
        Rectangle {
            point: Point::new(x.into(), y.into()),
            size: Size::new(w.into(), h.into())
        }
    }

    /// Convenience function for constructing a Rectangle with all fields
    /// set to zero.
    pub fn zeroed() -> Self {
        Rectangle::new(0, 0, 0, 0)
    }

    /// Creates a `Geometry` based at the origin (0, 0)
    /// with the given dimensions `height` and `width`.
    pub fn at_origin(height: i32, width: i32) -> Self {
        Self::new(0, 0, height, width)
    }

    /// Check whether this geometry encloses another geometry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Rectangle;
    ///
    /// let original = Geometry::new(0, 0, 100, 200);
    ///
    /// let new = Geometry::new(2, 2, 50, 75);
    ///
    /// assert!(original.contains(&new));
    /// ```
    pub fn contains(&self, other: &Rectangle<Kind>) -> bool {
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

    /// Check whether this geometry contains a certain point.
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::{Geometry, Point};
    ///
    /// let original = Geometry::new(0, 0, 100, 200);
    ///
    /// let point = Point::new(50, 50);
    ///
    /// assert!(original.contains_point(point));
    /// ```
    pub fn contains_point(&self, pt: Point<Kind>) -> bool {
        let wrange = self.point.x..(self.point.x + self.size.width);
        let hrange = self.point.y..(self.point.y + self.size.height);

        wrange.contains(&pt.x) && hrange.contains(&pt.y)
    }

    /// Splits a Geometry into `n` parts horizontally, each part
    /// covering a region of the original Geometry, top down.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Geometry;
    ///
    /// let original = Geometry::new(0, 0, 100, 200);
    ///
    /// let new_geoms = original.split_horz_n(2);
    ///
    /// assert_eq!(new_geoms, vec![
    ///     Geometry::new(0, 0, 50, 200),
    ///     Geometry::new(0, 50, 50, 200),
    /// ]);
    /// ```
    #[must_use]
    pub fn split_horz_n(&self, n: usize) -> Vec<Self> {
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

    /// Splits a Geometry into `n` parts vertically, each part
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

    /// Trim off an area from a Geometry from the side corresponding
    /// to `dir` (`Cardinal::Up` trims the top, `CardinaL::Down`
    /// trims the bottom).
    ///
    /// This returns a new Geometry.
    #[must_use]
    pub fn trim(&self, trim: i32, dir: Cardinal) -> Self {
        use Cardinal::*;
        match dir {
            Up => Rectangle::new(self.point.x, self.point.y + trim, self.size.height - trim, self.size.width),
            Down => Rectangle::new(self.point.x, self.point.y, self.size.height - trim, self.size.width),
            Left => Rectangle::new(self.point.x + trim, self.point.y, self.size.height, self.size.width - trim),
            Right => Rectangle::new(self.point.x, self.point.y, self.size.height, self.size.width - trim),
        }
    }

    /// Creates a new Geometry offset by `delta` pixels in the given
    /// direction `dir` (unidirectional offset).
    pub fn unidir_offset(&self, delta: i32, dir: Cardinal) -> Self {
        let Rectangle {point, size} = *self;

        let point = point.unidir_offset(delta, dir);

        Rectangle {point, size}
    }

    /// Creates a new Geometry offset by `dx, dy` pixels in the given
    /// directions `dirx, diry` (bidirectional offset).
    pub fn bidir_offset(&self, dx: i32, dy: i32, dirx: CardinalX, diry: CardinalY) -> Self {
        let Rectangle {point, size} = *self;

        let point = point.bidir_offset(dx, dy, dirx, diry);

        Rectangle {point, size}
    }

    /// Returns a Rectangle formed by the intersection of another Geometry.
    /// This is effectively a set containing all points found in both Geometries.
    pub fn intersect(&self, _other: Rectangle<Kind>) -> Self {
        todo!()
    }

    /// Returns a Rectangle by a given scale factor for the x and y axes.
    /// Also scales the Rectangle's position with respect to the origin (0, 0).
    pub fn scale(&self, scale_x: f32, scale_y: f32) -> Self {
        self.scale_gen::<Kind>(scale_x, scale_y)
    }

    fn scale_gen<K: GeometryKind>(&self, scale_x: f32, scale_y: f32) -> Rectangle<K> {
        Rectangle {
            point: self.point.scale_gen::<K>(scale_x, scale_y),
            size: self.size.scale_gen::<K>(scale_x, scale_y),
        }
    }
}

impl Rectangle<Logical> {
    /// Returns a `Rectangle<Physical>`, scaled by `scale`.
    pub fn as_physical(&self, scale: f32) -> Rectangle<Physical> {
        self.scale_gen::<Physical>(scale, scale)
    }
}

impl Rectangle<Physical> {
    /// Returns a `Rectangle<Logical>`, scaled by **`1 / scale`.**
    pub fn as_logical(&self, scale: f32) -> Rectangle<Logical> {
        let inverse = if scale == 0. {0.} else {scale.recip()};
        self.scale_gen::<Logical>(inverse, inverse)
    }
}

/// A representation of a color, following the RGBA model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color(u32);

impl Color {
    /// Creates the Color from a 32-bit integer.
    pub fn from_hex(hex: u32) -> Self {
        Self(hex)
    }

    /// Expresses the Color as a hex string.
    pub fn as_string(&self) -> String {
        format!("{:#x}", self.as_u32())
    }

    /// Returns the (R, G, B) values of the Color
    /// as bytes.
    pub fn rgb(&self) -> (u8, u8, u8) {
        let (r, g, b, _) = self.rgba();
        (r, g, b)
    }

    /// Returns the (R, G, B, A) values of the Color
    /// as bytes.
    pub fn rgba(&self) -> (u8, u8, u8, u8) {
        let [r, g, b, a] = u32::to_be_bytes(self.0);
        (r, g, b, a)
    }

    /// Returns the (R, G, B) components of the Color
    /// as proportions of max intensity (255.0).
    pub fn rgb_f32(&self) -> (f32, f32, f32) {
        let (r, g, b, _) = self.rgba_f32();
        (r, g, b)
    }

    /// Returns the (R, G, B, A) components of the Color
    /// as proportions of max intensity (255.0).
    pub fn rgba_f32(&self) -> (f32, f32, f32, f32) {
        let (r, g, b, a) = self.rgba();

        (
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Returns the color as a u32.
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Color {
    fn from(from: u32) -> Self {
        Self::from_hex(from)
    }
}

/// Whether the mouse button is pressed, and what state the mouse is in
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum MouseMode {
    None,
    Move,
    Resize,
}

/// Determines the colour that should be applied to
/// the window border.
///
/// The actual colour values are specified in `Config`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderStyle {
    /// The colour to be applied to the focused window.
    Focused,
    /// The colour to be applied to an unfocused window.
    Unfocused,
    /// The colour to applied when a window is marked as urgent.
    Urgent,
}
