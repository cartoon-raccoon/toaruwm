//! Types for configuring a `WindowManager`.
use custom_debug_derive::Debug;

use crate::{ToaruError::*, Result};
use crate::types::Color;
use crate::core::{Ring, WorkspaceSpec};
use crate::layouts::{Layout, Layouts, Floating, DynamicTiled};

/// Traits that `Config` is generic over.
pub mod traits {
    /// A type that knows its exact length.
    pub trait Length {
        /// The length of the item.
        fn len(&self) -> usize;
    }
}

pub use traits::Length;


/// Configuration of a window manager.
/// 
/// It is heavily generic over various types.
/// 
/// There are a few invariants related to a Configuration
/// that must always be upheld:
/// - `workspace` and `layouts` must never be empty.
/// - `main_ratio_inc` should always be > 0.
/// 
/// To this end, runtime checks are in place to ensure that
/// these invariants are upheld.
/// 
/// # Example
/// 
/// ```rust
/// # use toaruwm::layouts::Layouts;
/// # use toaruwm::types::Color;
/// use toaruwm::Config;
/// 
/// let config = Config {
///    workspaces: vec![
///        WorkspaceSpec::new("1", 0, layouts.clone()),
///        WorkspaceSpec::new("2", 0, layouts.clone()),
///        WorkspaceSpec::new("3", 0, layouts.clone()),
///    ],
///    gap_px: 0,
///    main_ratio_inc: 0.05,
///    layouts: Layouts::with_layouts_validated(
///        vec![
///            Box::new(DynamicTiled::new(0.5, 2)) as Box<dyn Layout>,
///            Box::new(Floating::new()) as Box<dyn Layout>,
///        ]
///    ).unwrap(),
///    float_classes: Vec::new(),
///    border_px: 2,
///    unfocused: Color::from(0x555555ff),
///    focused: Color::from(0xddddddff),
///    urgent: Color::from(0xee0000ff),
/// };
/// 
/// config.validate().expect("invalid config");
/// ```
/// 
#[derive(Debug)]
pub struct Config<W, L, F>
where
    W: IntoIterator<Item = WorkspaceSpec> + Length,
    L: IntoIterator<Item = Box<dyn Layout>> + Length,
    F: IntoIterator<Item = String>,
{
    /// The workspaces and the screen it should be sent to.
    /// (Name, Screen)
    pub workspaces: W,
    /// The gap between windows.
    pub gap_px: u32,
    /// When the main ratio is changed, by what increment?
    pub main_ratio_inc: f64,
    /// The set of layouts being used.
    pub layouts: L,
    /// The window classes that should float.
    pub float_classes: F,
    /// The width of the window border.
    pub border_px: u32,
    /// The color to apply to the borders of an unfocused window.
    pub unfocused: Color,
    /// The color to apply to the borders of a focused window.
    pub focused: Color,
    /// The color to apply to the borders of a window marked as urgent.
    pub urgent: Color,
}

impl<W, L, F> Config<W, L, F>
where
    W: IntoIterator<Item = WorkspaceSpec> + Length,
    L: IntoIterator<Item = Box<dyn Layout>> + Length,
    F: IntoIterator<Item = String>,
{
    /// Checks the configuration to verify that all invariants are upheld.
    pub fn validate(&self) -> Result<()> {
        if self.workspaces.len() < 1 {
            return Err(InvalidConfig("workspaces is empty".into()))
        }
        if self.layouts.len() < 1 {
            return Err(InvalidConfig("layouts is empty".into()))
        }
        if self.main_ratio_inc < 0.0 {
            return Err(InvalidConfig(
                format!("main_ratio_inc < 0: = {}", self.main_ratio_inc)
            ))
        }
        Ok(())
    }
}

impl Default for Config<Vec<WorkspaceSpec>, Layouts, Vec<String>> {
    fn default() -> Config<Vec<WorkspaceSpec>, Layouts, Vec<String>> {
        let layouts = vec![String::from("DTiled"), String::from("Floating"), ];
        Config {
            workspaces: vec![
                WorkspaceSpec::new("1", 0, layouts.clone()),
                WorkspaceSpec::new("2", 0, layouts.clone()),
                WorkspaceSpec::new("3", 0, layouts.clone()),
            ],
            gap_px: 0,
            main_ratio_inc: 0.05,
            layouts: Layouts::with_layouts_validated(
                vec![
                    Box::new(DynamicTiled::new(0.5, 2)) as Box<dyn Layout>,
                    Box::new(Floating::new()) as Box<dyn Layout>,
                ]
            ).unwrap(),
            float_classes: Vec::new(),
            border_px: 2,
            unfocused: Color::from(0x555555ff),
            focused: Color::from(0xddddddff),
            urgent: Color::from(0xee0000ff),
        }
    }
}

use std::collections::{VecDeque, LinkedList};

macro_rules! _impl_length {
    ($t:ty) => {
        impl<T> Length for $t {
            fn len(&self) -> usize {
                self.len()
            }
        }
    }
}

_impl_length!(Vec<T>);
_impl_length!(Ring<T>);
_impl_length!(VecDeque<T>);
_impl_length!(LinkedList<T>);


//todo: add validation, builder, etc
