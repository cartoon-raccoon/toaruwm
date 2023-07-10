use std::ops::Deref;

use tracing::error;

use crate::layouts::LayoutType;

pub use crate::core::{Ring, Selector};
use crate::x::{
    core::{StackMode, XAtom, XConn},
    property::{WindowState, WmHints, WmSizeHints},
};

pub use super::window::{Client, ClientRing};

// todo: deprecate this and put inside configuration
pub const BORDER_WIDTH: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

/// A cardinal direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cardinal {
    Up,
    Down,
    Left,
    Right,
}

/// A type for representing a point on a display or screen.
///
/// Implements [`PartialEq`][1], so you can compare it directly with
/// another Point.
///
/// [1]: std::cmp::PartialEq
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
}

/// A type for representing a 2D rectangular space on a display or screen.
///
/// Implements [`PartialEq`][1], so you can compare it directly with
/// another Geometry.
///
/// _Note:_ The Default impl returns
/// Geometry {0, 0, 100, 160}, **NOT** zeroed.
///
/// [1]: std::cmp::PartialEq
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    // The x coordinate of the top left corner.
    pub x: i32,
    // The y coordinate of the top left corner.
    pub y: i32,
    // The height of the geometry.
    pub height: i32,
    // The width of the geometry.
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
            Geometry { x, width, .. } if (*x + *width as i32) > (self.x + self.width as i32) => {
                false
            }
            Geometry { y, .. } if *y < self.y => false,
            Geometry { y, height, .. } if (*y + *height as i32) > (self.y + self.height as i32) => {
                false
            }
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
        let wrange = self.x..(self.x + self.width as i32);
        let hrange = self.y..(self.y + self.height as i32);

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
                y: self.y + (i as i32 * new_height as i32),
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

        for i in 0..n as usize {
            ret.push(Geometry {
                x: self.x + (i as i32 * new_width as i32),
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
                y: self.y + top_height as i32,
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
                x: self.x + left_width as i32,
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
                y: self.y + height as i32,
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
                x: self.x + width as i32,
                y: self.y,
                height: self.height,
                width: self.width - width,
            },
        )
    }

    /// Trim off an area from a Geometry.
    ///
    /// This returns a new geometry.
    #[must_use]
    pub fn trim(&self, trim: i32, dir: Cardinal) -> Geometry {
        use Cardinal::*;
        match dir {
            Up => Geometry::new(self.x - trim, self.y, self.width, self.height - trim),
            Down => Geometry::new(self.x, self.y, self.width, self.height - trim),
            Left => Geometry::new(self.x, self.y + trim, self.width - trim, self.height),
            Right => Geometry::new(self.x, self.y, self.width - trim, self.height),
        }
    }
}

// Whether the mouse button is pressed.
#[derive(Debug, Clone, Copy)]
pub enum MouseMode {
    None,
    Move,
    Resize,
}

/// The layout state of the Window.
#[derive(Clone, Copy, Debug)]
pub(crate) enum WinLayoutState {
    Tiled,
    Floating,
}

/// Determines the colour that should be applied to
/// the window border.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderStyle {
    Focused,
    Unfocused,
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
    Resize { h: i32, w: i32 },
    /// Moving the window.
    Move { x: i32, y: i32 },
    /// Stacking mode of the window.
    StackingMode(StackMode),
}

/// Attribute options for a Client.
#[derive(Clone, Copy, Debug)]
pub enum ClientAttrs {
    /// The colour of the border.
    BorderColour(BorderStyle),
    /// Client event mask.
    EnableClientEvents,
    /// Disable client events.
    DisableClientEvents,
    /// Root window attributes required for the WM to work.
    RootEventMask,
}

/// Convenience wrapper around a Vec of NetWindowStates.
#[derive(Debug, Clone, Default)]
pub struct NetWindowStates {
    states: Vec<XAtom>,
}

impl NetWindowStates {
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    pub fn from_strings<X: XConn>(strs: Vec<String>, conn: &X) -> Self {
        strs.into_iter()
            .map(|s| conn.atom(&s))
            .filter(|r| r.is_ok()) // filter out errors
            .map(|a| a.unwrap()) // safe to unwrap since errors filtered out
            .collect::<Vec<XAtom>>()
            .into()
    }

    pub fn contains(&self, prop: XAtom) -> bool {
        self.states.contains(&prop)
    }

    pub fn add(&mut self, prop: XAtom) {
        self.states.push(prop)
    }

    pub fn remove(&mut self, prop: XAtom) -> XAtom {
        for (idx, atom) in self.states.iter().enumerate() {
            if *atom == prop {
                return self.states.remove(idx);
            }
        }
        error!("Tried to remove atom not in states");
        0
    }
}

impl From<Vec<XAtom>> for NetWindowStates {
    fn from(from: Vec<XAtom>) -> Self {
        Self { states: from }
    }
}

impl Deref for NetWindowStates {
    type Target = [XAtom];

    fn deref(&self) -> &Self::Target {
        self.states.as_slice()
    }
}

impl From<LayoutType> for WinLayoutState {
    #[inline]
    fn from(from: LayoutType) -> WinLayoutState {
        if let LayoutType::Floating = from {
            return Self::Floating;
        }

        Self::Tiled
    }
}

/// ICCCM-defined window properties.
//todo: make all fields private, accessible with methods.
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
    pub fn wm_name(&self) -> &str {
        &self.wm_name
    }

    pub fn wm_icon_name(&self) -> &str {
        &self.wm_icon_name
    }

    #[inline]
    pub fn wm_size_hints(&self) -> Option<&WmSizeHints> {
        self.wm_size_hints.as_ref()
    }

    pub fn wm_hints(&self) -> Option<&WmHints> {
        self.wm_hints.as_ref()
    }

    pub fn wm_class(&self) -> (&str, &str) {
        (&self.wm_class.0, &self.wm_class.1)
    }

    pub fn window_type(&self) -> Option<&[XAtom]> {
        self.wm_protocols.as_deref()
    }

    pub fn wm_state(&self) -> Option<WindowState> {
        self.wm_state
    }
}
