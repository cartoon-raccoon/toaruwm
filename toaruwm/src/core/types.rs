//! Basic core types used throughout this crate at a high level.
//!
//! # Important Note
//!
//! For types pertaining to the cartesian plane,
//! all of them assume the default X server window gravity (NorthWest).
//!
//! For X server-specific types, see [`crate::x::core`].

use std::ops::Deref;

use tracing::error;

#[doc(inline)]
pub use crate::core::{Ring, Selector};
use crate::x::{
    core::{StackMode, XAtom, XConn},
    property::{WindowState, WmHints, WmSizeHints},
};

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

/// A subset of Cardinal with only Up and Down variants.
///
/// It is disjoint with `CardinalX`.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardinalY {
    Up,
    Down,
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
    /// Dead center in the Geometry.
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
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
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
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
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
    pub fn calculate_offset(&self, other: Point) -> (i32, i32) {
        (other.x - self.x, other.y - self.y)
    }

    /// Calculates the distance to another point, using the
    /// Pythagorean theorem.
    ///
    /// Since most things in this crate take integer values,
    /// you will probably want to round this up/down to
    /// the nearest integer value before coercing to an
    /// integer type.
    pub fn distance_to(&self, other: Point) -> f64 {
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

        let Point { x, y } = *self;

        match dir {
            Up => Point { x, y: y - delta },
            Down => Point { x, y: y + delta },
            Left => Point { x: x - delta, y },
            Right => Point { x: x + delta, y },
        }
    }

    /// Creates a Point offset by `dx, dy` in the given directions
    /// (bidirectional offset).
    ///
    /// Moves the point in both directions.
    pub fn bidir_offset(&self, dx: i32, dy: i32, dirx: CardinalX, diry: CardinalY) -> Self {
        use CardinalX::*;
        use CardinalY::*;

        let Point { x, y } = *self;

        match (dirx, diry) {
            (Left, Up) => Point {
                x: x - dx,
                y: y - dy,
            },
            (Left, Down) => Point {
                x: x - dx,
                y: y + dy,
            },
            (Right, Down) => Point {
                x: x + dx,
                y: y + dy,
            },
            (Right, Up) => Point {
                x: x + dx,
                y: y - dy,
            },
        }
    }

    /// Offsets itself by `delta` in the given direction.
    pub fn unidir_offset_in_place(&mut self, delta: i32, dir: Cardinal) {
        let Point { x, y } = self.unidir_offset(delta, dir);

        self.x = x;
        self.y = y;
    }

    /// Offsets itself by `dx, dy` in the given directions.
    pub fn bidir_offset_in_place(&mut self, dx: i32, dy: i32, dirx: CardinalX, diry: CardinalY) {
        let Point { x, y } = self.bidir_offset(dx, dy, dirx, diry);

        self.x = x;
        self.y = y;
    }
}

/// A type for representing a 2D rectangular space on a display or screen.
///
/// Implements [`PartialEq`][1], so you can compare it directly with
/// another Geometry.
///
/// # Note on X Window Gravity
///
/// Geometries follow the X Window System default
/// of taking their gravity from the top-left corner,
/// that is, (0, 0) is considered the top left corner
/// of the screen, and any increase is an offset to the right
/// or downwards.
///
/// _Note:_ The Default impl returns
/// Geometry {0, 0, 100, 160}, **NOT** zeroed.
///
/// [1]: std::cmp::PartialEq
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    /// The x coordinate of the top left corner.
    pub x: i32,
    /// The y coordinate of the top left corner.
    pub y: i32,
    /// The height of the geometry.
    pub height: i32,
    /// The width of the geometry.
    pub width: i32,
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            x: 0,
            y: 0,
            height: 100,
            width: 160,
        }
    }
}

impl Geometry {
    /// Constructs a new Geometry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Geometry;
    ///
    /// let geom1 = Geometry::new(0, 0, 100, 160);
    /// let geom2 = Geometry {
    ///     x: 0,
    ///     y: 0,
    ///     height: 100,
    ///     width: 160,
    /// };
    ///
    /// assert_eq!(geom1, geom2);
    /// ```
    pub fn new(x: i32, y: i32, h: i32, w: i32) -> Self {
        Geometry {
            x,
            y,
            height: h,
            width: w,
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
        Geometry {
            x: 0,
            y: 0,
            height: 0,
            width: 0,
        }
    }

    /// Creates a `Geometry` based at the origin (0, 0)
    /// with the given dimensions `height` and `width`.
    pub fn at_origin(height: i32, width: i32) -> Geometry {
        Self {
            x: 0,
            y: 0,
            height,
            width,
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
    pub fn contains(&self, other: &Geometry) -> bool {
        match other {
            Geometry { x, .. } if *x < self.x => false,
            Geometry { x, width, .. } if (*x + *width) > (self.x + self.width) => false,
            Geometry { y, .. } if *y < self.y => false,
            Geometry { y, height, .. } if (*y + *height) > (self.y + self.height) => false,
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
    pub fn contains_point(&self, pt: Point) -> bool {
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
    pub fn split_horz_n(&self, n: usize) -> Vec<Geometry> {
        let new_height = self.height / n as i32;

        let mut ret = Vec::new();

        for i in 0..n {
            ret.push(Geometry {
                x: self.x,
                y: self.y + (i as i32 * new_height),
                height: new_height,
                width: self.width,
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
    pub fn split_vert_n(&self, n: usize) -> Vec<Geometry> {
        let new_width = self.width / n as i32;

        let mut ret = Vec::new();

        for i in 0..n {
            ret.push(Geometry {
                x: self.x + (i as i32 * new_width),
                y: self.y,
                height: self.height,
                width: new_width,
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
    pub fn split_horz_ratio(&self, ratio: f32) -> (Geometry, Geometry) {
        let ratio = ratio.clamp(0.0, 1.0);

        if ratio.is_nan() {
            panic!("Got f32::NAN");
        }

        let top_height = (self.height as f32 * ratio) as i32;
        let bottom_height = self.height - top_height;

        (
            // top
            Geometry {
                x: self.x,
                y: self.y,
                height: top_height,
                width: self.width,
            },
            // bottom
            Geometry {
                x: self.x,
                y: self.y + top_height,
                height: bottom_height,
                width: self.width,
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
    pub fn split_vert_ratio(&self, ratio: f32) -> (Geometry, Geometry) {
        let ratio = ratio.clamp(0.0, 1.0);

        if ratio.is_nan() {
            panic!("Got f32::NAN");
        }

        let left_width = (self.width as f32 * ratio) as i32;
        let right_width = self.width - left_width;

        (
            // left
            Geometry {
                x: self.x,
                y: self.y,
                height: self.height,
                width: left_width,
            },
            // right
            Geometry {
                x: self.x + left_width,
                y: self.y,
                height: self.height,
                width: right_width,
            },
        )
    }

    /// Splits a Geometry _horizontally_ into two Geometries
    /// at a given height.
    ///
    /// Returns (top, bottom), where bottom has the given height.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Geometry;
    ///
    /// let original = Geometry::new(0, 0, 100, 200);
    ///
    /// let (top, bottom) = original.split_at_height(60);
    ///
    /// assert_eq!(top, Geometry::new(0, 0, 40, 200));
    /// assert_eq!(bottom, Geometry::new(0, 60, 60, 200));
    /// ```
    #[must_use]
    pub fn split_at_height(&self, height: i32) -> (Geometry, Geometry) {
        (
            // Top
            Geometry {
                x: self.x,
                y: self.y,
                height: self.height - height,
                width: self.width,
            },
            // Bottom
            Geometry {
                x: self.x,
                y: self.y + height,
                height,
                width: self.width,
            },
        )
    }

    /// Splits a Geometry _horizontally_ into two Geometries
    /// at a given width.
    ///
    /// Returns (left, right), where left has the given width.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::Geometry;
    ///
    /// let original = Geometry::new(0, 0, 100, 200);
    ///
    /// let (left, right) = original.split_at_width(120);
    ///
    /// assert_eq!(left, Geometry::new(0, 0, 100, 120));
    /// assert_eq!(right, Geometry::new(120, 0, 100, 80));
    /// ```
    #[must_use]
    pub fn split_at_width(&self, width: i32) -> (Geometry, Geometry) {
        (
            // Left
            Geometry {
                x: self.x,
                y: self.y,
                height: self.height,
                width,
            },
            // Right
            Geometry {
                x: self.x + width,
                y: self.y,
                height: self.height,
                width: self.width - width,
            },
        )
    }

    /// Trim off an area from a Geometry from the side corresponding
    /// to `dir` (`Cardinal::Up` trims the top, `CardinaL::Down`
    /// trims the bottom).
    ///
    /// This returns a new Geometry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::types::{Geometry, Cardinal::*};
    ///
    /// let g1 = Geometry::new(0, 0, 100, 160);
    /// /* trim off 5 from the left and 10 from the top */
    /// let g2 = g1.trim(5, Left).trim(10, Up);
    ///
    /// assert_eq!(g2, Geometry {
    ///     x: 5, y: 10, height: 90, width: 155
    /// });
    /// ```
    #[must_use]
    pub fn trim(&self, trim: i32, dir: Cardinal) -> Geometry {
        use Cardinal::*;
        match dir {
            Up => Geometry::new(self.x, self.y + trim, self.height - trim, self.width),
            Down => Geometry::new(self.x, self.y, self.height - trim, self.width),
            Left => Geometry::new(self.x + trim, self.y, self.height, self.width - trim),
            Right => Geometry::new(self.x, self.y, self.height, self.width - trim),
        }
    }

    /// Creates a new Geometry offset by `delta` pixels in the given
    /// direction `dir` (unidirectional offset).
    pub fn unidir_offset(&self, delta: i32, dir: Cardinal) -> Geometry {
        let Geometry {
            x,
            y,
            height,
            width,
        } = *self;

        let Point { x, y } = Point { x, y }.unidir_offset(delta, dir);

        Geometry {
            x,
            y,
            height,
            width,
        }
    }

    /// Creates a new Geometry offset by `dx, dy` pixels in the given
    /// directions `dirx, diry` (bidirectional offset).
    pub fn bidir_offset(&self, dx: i32, dy: i32, dirx: CardinalX, diry: CardinalY) -> Geometry {
        let Geometry {
            x,
            y,
            height,
            width,
        } = *self;

        let Point { x, y } = Point { x, y }.bidir_offset(dx, dy, dirx, diry);

        Geometry {
            x,
            y,
            height,
            width,
        }
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

/// Configuration options for a Client.
#[derive(Clone, Copy, Debug)]
pub enum ClientConfig {
    /// Width of the window border.
    BorderWidth(u32),
    /// Position of the window.
    Position(Geometry),
    /// Resizing the window.
    Resize {
        /// The height.
        h: i32,
        /// The width.
        w: i32,
    },
    /// Moving the window.
    #[allow(missing_docs)]
    Move { x: i32, y: i32 },
    /// Stacking mode of the window.
    StackingMode(StackMode),
}

/// Attribute options for a Client.
#[derive(Clone, Copy, Debug)]
pub enum ClientAttrs {
    /// The colour of the border.
    BorderColour(Color),
    /// Client event mask.
    EnableClientEvents,
    /// Disable client events.
    DisableClientEvents,
    /// Root window attributes required for the WM to work.
    RootEventMask,
}

/// Convenience wrapper around a list of NetWindowStates.
#[derive(Debug, Clone, Default)]
pub struct NetWindowStates {
    states: Vec<XAtom>,
}

impl NetWindowStates {
    /// Creates a new `NetWindowStates`.
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    /// Creates a new `NetWindowStates from an iterator of types.
    pub fn from_strings<I, X: XConn>(strs: I, conn: &X) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        Self {
            states: strs
                .into_iter()
                .map(|s| conn.atom(&s))
                .filter_map(|a| a.ok()) // filter out errors
                .collect::<Vec<XAtom>>(),
        }
    }

    /// Checks whether `self` contains the given atom.
    pub fn contains(&self, prop: XAtom) -> bool {
        self.states.contains(&prop)
    }

    /// Adds a new atom to `self`.
    pub fn add(&mut self, prop: XAtom) {
        self.states.push(prop)
    }

    /// Removes a given atom.
    pub fn remove(&mut self, prop: XAtom) -> XAtom {
        for (idx, atom) in self.states.iter().enumerate() {
            if *atom == prop {
                return self.states.remove(idx);
            }
        }
        error!("Tried to remove atom not in states");
        XAtom::from(0)
    }
}

impl<I> From<I> for NetWindowStates
where
    I: Iterator<Item = XAtom>,
{
    fn from(from: I) -> Self {
        Self {
            states: from.collect(),
        }
    }
}

impl Deref for NetWindowStates {
    type Target = [XAtom];

    fn deref(&self) -> &Self::Target {
        self.states.as_slice()
    }
}

impl IntoIterator for NetWindowStates {
    type Item = XAtom;
    type IntoIter = std::vec::IntoIter<XAtom>;

    fn into_iter(self) -> Self::IntoIter {
        self.states.into_iter()
    }
}

/// ICCCM-defined window properties.
//todo: make all fields private, accessible with methods.
#[derive(Clone, Debug)]
pub struct XWinProperties {
    pub(crate) wm_name: String,
    pub(crate) wm_icon_name: String,
    pub(crate) wm_size_hints: Option<WmSizeHints>,
    pub(crate) wm_hints: Option<WmHints>,
    pub(crate) wm_class: (String, String), //Instance, Class
    pub(crate) wm_protocols: Option<Vec<XAtom>>,
    pub(crate) wm_state: Option<WindowState>,
}

impl XWinProperties {
    /// Returns `WM_NAME`.
    pub fn wm_name(&self) -> &str {
        &self.wm_name
    }
    /// Returns `WM_ICON_NAME`.
    pub fn wm_icon_name(&self) -> &str {
        &self.wm_icon_name
    }
    /// Returns `WM_SIZE_HINTS`, if set.
    #[inline]
    pub fn wm_size_hints(&self) -> Option<&WmSizeHints> {
        self.wm_size_hints.as_ref()
    }
    /// Returns `WM_HINTS`, if set.
    pub fn wm_hints(&self) -> Option<&WmHints> {
        self.wm_hints.as_ref()
    }
    /// Returns `WM_CLASS`, it set.
    pub fn wm_class(&self) -> (&str, &str) {
        (&self.wm_class.0, &self.wm_class.1)
    }
    /// Returns a list of window types.
    pub fn window_type(&self) -> Option<&[XAtom]> {
        self.wm_protocols.as_deref()
    }
    /// Returns the state of the window.
    pub fn wm_state(&self) -> Option<WindowState> {
        self.wm_state
    }
}
