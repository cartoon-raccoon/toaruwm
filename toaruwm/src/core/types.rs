//! Basic core types used throughout this crate at a high level.
//!
//! # Important Note
//!
//! For types pertaining to the cartesian plane,
//! all of them assume the default X server window gravity (NorthWest).
//!
//! For X server-specific types, see [`crate::x::core`].


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
    pub trait GeometryKind: private::Sealed {}

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

    /// Calculates the distance to another point, using the
    /// Pythagorean theorem.
    ///
    /// Since most things in this crate take integer values,
    /// you will probably want to round this up/down to
    /// the nearest integer value before coercing to an
    /// integer type.
    pub fn distance_to(&self, other: Point<Kind>) -> f64 {
        let (x, y) = self.calculate_offset(other);

        let ret = ((x as f64).powi(2) + (y as f64).powi(2)).sqrt();

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

    fn scale_gen<K: GeometryKind>(&self, scale_x: f32, scale_y: f32) -> Point<K> {
        let Point { x, y, .. } = *self;

        Point {
            x: ((x as f32) * scale_x).round() as i32,
            y: ((y as f32) * scale_y).round() as i32,
            _kind: PhantomData,
        }
    }
}

impl Point<Logical> {
    /// Returns a `Rectangle<Physical>`, scaled by `scale`.
    pub fn as_physical(&self, scale: f32) -> Point<Physical> {
        self.scale_gen::<Physical>(scale, scale)
    }
}

impl Point<Physical> {
    /// Returns a `Rectangle<Logical>`, scaled by **`1 / scale`.**
    pub fn as_logical(&self, scale: f32) -> Point<Logical> {
        // account for if scale == 0, since calling recip on 0 will give
        // a divide-by-zero error
        let inverse = if scale == 0. {0.} else {scale.recip()};
        self.scale_gen::<Logical>(inverse, inverse)
    }
}

/// A type for representing a 2D rectangular space on a display or screen.
///
/// Implements [`PartialEq`][1], so you can compare it directly with
/// another Geometry.
///
/// # Note on X Window Gravity
///
/// Rectangles follow the X Window System default
/// of taking their gravity from the top-left corner,
/// that is, (0, 0) is considered the top left corner
/// of the screen, and any increase is an offset to the right
/// or downwards.
///
/// _Note:_ The Default impl returns Rectangle {0, 0, 0, 0}.
///
/// [1]: std::cmp::PartialEq
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle<Kind: GeometryKind> {
    /// The x coordinate of the top left corner.
    pub x: i32,
    /// The y coordinate of the top left corner.
    pub y: i32,
    /// The height of the geometry.
    pub height: i32,
    /// The width of the geometry.
    pub width: i32,

    _kind: PhantomData<Kind>
}

impl<Kind: GeometryKind> Default for Rectangle<Kind> {
    fn default() -> Self {
        Rectangle {
            x: 0,
            y: 0,
            height: 100,
            width: 160,

            _kind: PhantomData
        }
    }
}

impl<Kind: GeometryKind> Rectangle<Kind> {
    /// Constructs a new Geometry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Rectangle;
    ///
    /// let geom1 = Rectangle::new(0, 0, 100, 160);
    /// let geom2 = Rectangle {
    ///     x: 0,
    ///     y: 0,
    ///     height: 100,
    ///     width: 160,
    /// };
    ///
    /// assert_eq!(geom1, geom2);
    /// ```
    pub fn new<N: Into<i32>>(x: N, y: N, h: N, w: N) -> Self {
        Rectangle {
            x: x.into(),
            y: y.into(),
            height: h.into(),
            width: w.into(),
            _kind: PhantomData,
        }
    }

    /// Convenience function for constructing a Geometry with all fields
    /// set to zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Geometry;
    ///
    /// let geom = Geometry::zeroed();
    ///
    /// assert_eq!(geom, Geometry::new(0, 0, 0, 0));
    /// ```
    pub fn zeroed() -> Self {
        Rectangle {
            x: 0,
            y: 0,
            height: 0,
            width: 0,
            _kind: PhantomData,
        }
    }

    /// Creates a `Geometry` based at the origin (0, 0)
    /// with the given dimensions `height` and `width`.
    pub fn at_origin(height: i32, width: i32) -> Self {
        Self {
            x: 0,
            y: 0,
            height,
            width,
            _kind: PhantomData,
        }
    }

    /// Check whether this geometry encloses another geometry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Geometry;
    ///
    /// let original = Geometry::new(0, 0, 100, 200);
    ///
    /// let new = Geometry::new(2, 2, 50, 75);
    ///
    /// assert!(original.contains(&new));
    /// ```
    pub fn contains(&self, other: &Rectangle<Kind>) -> bool {
        match other {
            Rectangle { x, .. } if *x < self.x => false,
            Rectangle { x, width, .. } if (*x + *width) > (self.x + self.width) => false,
            Rectangle { y, .. } if *y < self.y => false,
            Rectangle { y, height, .. } if (*y + *height) > (self.y + self.height) => false,
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
        let wrange = self.x..(self.x + self.width);
        let hrange = self.y..(self.y + self.height);

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
        let new_height = self.height / n as i32;

        let mut ret = Vec::new();

        for i in 0..n {
            ret.push(Rectangle {
                x: self.x,
                y: self.y + (i as i32 * new_height),
                height: new_height,
                width: self.width,
                _kind: PhantomData,
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
        let new_width = self.width / n as i32;

        let mut ret = Vec::new();

        for i in 0..n {
            ret.push(Rectangle {
                x: self.x + (i as i32 * new_width),
                y: self.y,
                height: self.height,
                width: new_width,
                _kind: PhantomData,
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

        let top_height = (self.height as f32 * ratio) as i32;
        let bottom_height = self.height - top_height;

        (
            // top
            Rectangle {
                x: self.x,
                y: self.y,
                height: top_height,
                width: self.width,
                _kind: PhantomData,
            },
            // bottom
            Rectangle {
                x: self.x,
                y: self.y + top_height,
                height: bottom_height,
                width: self.width,
                _kind: PhantomData,
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

        let left_width = (self.width as f32 * ratio) as i32;
        let right_width = self.width - left_width;

        (
            // left
            Rectangle {
                x: self.x,
                y: self.y,
                height: self.height,
                width: left_width,
                _kind: PhantomData,
            },
            // right
            Rectangle {
                x: self.x + left_width,
                y: self.y,
                height: self.height,
                width: right_width,
                _kind: PhantomData,
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
                x: self.x,
                y: self.y,
                height: self.height - height,
                width: self.width,
                _kind: PhantomData,
            },
            // Bottom
            Rectangle {
                x: self.x,
                y: self.y + height,
                height,
                width: self.width,
                _kind: PhantomData,
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
                x: self.x,
                y: self.y,
                height: self.height,
                width,
                _kind: PhantomData,
            },
            // Right
            Rectangle {
                x: self.x + width,
                y: self.y,
                height: self.height,
                width: self.width - width,
                _kind: PhantomData,
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
            Up => Rectangle::new(self.x, self.y + trim, self.height - trim, self.width),
            Down => Rectangle::new(self.x, self.y, self.height - trim, self.width),
            Left => Rectangle::new(self.x + trim, self.y, self.height, self.width - trim),
            Right => Rectangle::new(self.x, self.y, self.height, self.width - trim),
        }
    }

    /// Creates a new Geometry offset by `delta` pixels in the given
    /// direction `dir` (unidirectional offset).
    pub fn unidir_offset(&self, delta: i32, dir: Cardinal) -> Self {
        let Rectangle {
            x,
            y,
            height,
            width,
            _kind,
        } = *self;

        let Point { x, y, _kind } = Point { x, y, _kind }.unidir_offset(delta, dir);

        Rectangle {
            x,
            y,
            height,
            width,
            _kind,
        }
    }

    /// Creates a new Geometry offset by `dx, dy` pixels in the given
    /// directions `dirx, diry` (bidirectional offset).
    pub fn bidir_offset(&self, dx: i32, dy: i32, dirx: CardinalX, diry: CardinalY) -> Self {
        let Rectangle {
            x,
            y,
            height,
            width,
            _kind
        } = *self;

        let Point { x, y, _kind } = Point { x, y, _kind }.bidir_offset(dx, dy, dirx, diry);

        Rectangle {
            x,
            y,
            height,
            width,
            _kind,
        }
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
        let Rectangle {x, y, height, width, .. } = *self;

        Rectangle {
            x: ((x as f32) * scale_x).round() as i32,
            y: ((y as f32) * scale_y).round() as i32,
            height: ((height as f32) * scale_y).round() as i32,
            width: ((width as f32) * scale_x).round() as i32,
            _kind: PhantomData,
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
