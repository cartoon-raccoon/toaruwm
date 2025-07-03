//! Basic core types used throughout this crate at a high level.

#[doc(inline)]
pub use crate::core::{Ring, Selector};

pub mod geometry;

pub use geometry::*;

use std::hash::Hash;
use std::fmt::Debug;
use std::collections::HashMap;
use std::any::Any;

/// Data about a given client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientData {
    geom: Rectangle<i32, Logical>,
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
    {$($key:expr => $val:expr),+,} => {
        {
            let mut __dict = Dict::new();
    
            $(
                __dict.insert(String::from($key), Box::new($val));
            )+
    
            __dict
        }
    };
}

/// A transformation on a generic 2D plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Transform {
    /// The identity transform (nothing happens).
    #[default]
    Ident,
    /// The plane is rotated by 90 degrees.
    Rot90,
    /// The plane is rotated by 180 degrees.
    Rot180,
    /// The plane is rotated by 270 degrees.
    Rot270,
    /// The plane is flipped vertically.
    Flipped,
    /// The plane is flipped and then rotated by 90 degrees.
    Flipped90,
    /// The plane is flipped and then rotated by 180 degrees.
    Flipped180,
    /// The plane is flipped and then rotated by 270 degrees.
    Flipped270
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

/// A color gradient.
/// 
/// A color gradient consists of a set of one or more colors
/// that are interpolated between when being shown.
#[derive(Debug, Clone)]
pub struct Gradient {
    colors: Vec<Color>
}

impl Gradient {
    /// Creates a new `Gradient`.
    pub fn new<I: IntoIterator<Item = Color>>(colors: I) -> Self {
        Self {
            colors: colors.into_iter().collect()
        }
    }

    /// Retrieves the colors that a present in this Gradient.
    pub fn colors(&self) -> &[Color] {
        &self.colors
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
