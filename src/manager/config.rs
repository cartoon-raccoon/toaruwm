//! Types for configuring a `WindowManager`.
use custom_debug_derive::Debug;

use crate::{ToaruError::*, Result};
use crate::types::Color;
use crate::core::WorkspaceSpec;
use crate::layouts::{Layout, Floating, DynamicTiled};


/// Configuration of a window manager.
/// 
/// There are a few invariants related to a Configuration
/// that must always be upheld:
/// - `workspace` and `layouts` must never be empty.
/// - `main_ratio_inc` should always be > 0.
/// 
/// To this end, runtime checks are in place to ensure that
/// these invariants are upheld.
/// 
/// To build a Config, use the `ConfigBuilder` type.
/// 
/// # Example
/// 
/// ```ignore //fixme
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
///    layouts: vec![
///        Box::new(DynamicTiled::new(0.5, 2)) as Box<dyn Layout>,
///        Box::new(Floating::new()) as Box<dyn Layout>,
///    ],
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
pub struct Config {
    /// The workspaces and the screen it should be sent to.
    /// (Name, Screen)
    pub(crate) workspaces: Vec<WorkspaceSpec>,
    /// The gap between windows.
    pub(crate) gap_px: u32,
    /// When the main ratio is changed, by what increment?
    pub(crate) main_ratio_inc: f64,
    /// The set of layouts being used.
    #[debug(skip)]
    pub(crate) layouts: Vec<Box<dyn Layout>>,
    /// The window classes that should float.
    pub(crate) float_classes: Vec<String>,
    /// The width of the window border.
    pub(crate) border_px: u32,
    /// The color to apply to the borders of an unfocused window.
    pub(crate) unfocused: Color,
    /// The color to apply to the borders of a focused window.
    pub(crate) focused: Color,
    /// The color to apply to the borders of a window marked as urgent.
    pub(crate) urgent: Color,
}

impl Config {
    /// Returns the default construction.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns a `ConfigBuiilder`.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

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

    // todo: add methods to view inner fields
}

impl Default for Config {
    fn default() -> Config {
        let layouts = vec![String::from("DTiled"), String::from("Floating"), ];
        Config {
            workspaces: vec![
                WorkspaceSpec::new("1", 0, layouts.clone()),
                WorkspaceSpec::new("2", 0, layouts.clone()),
                WorkspaceSpec::new("3", 0, layouts.clone()),
            ],
            gap_px: 0,
            main_ratio_inc: 0.05,
            layouts: vec![
                Box::new(DynamicTiled::new(0.5, 2)) as Box<dyn Layout>,
                Box::new(Floating::new()) as Box<dyn Layout>,
            ],
            float_classes: Vec::new(),
            border_px: 2,
            unfocused: Color::from(0x555555ff),
            focused: Color::from(0xddddddff),
            urgent: Color::from(0xee0000ff),
        }
    }
}

/// A helper type to construct a `Config`.
//todo: add example
#[derive(Debug)]
pub struct ConfigBuilder {
    inner: Config,
}

impl ConfigBuilder {
    /// Creates a new `ConfigBuilder`.
    pub fn new() -> Self {
        Self {
            inner: Config::default()
        }
    }

    /// Sets the workspaces used by the WindowManager.
    pub fn workspaces<W>(mut self, workspaces: W) -> Self
    where
        W: IntoIterator<Item=WorkspaceSpec>
    {
        self.inner.workspaces = workspaces.into_iter().collect();
        self
    }

    /// Sets the layouts used by the WindowManager.
    pub fn layouts<L>(mut self, layouts: L) -> Self
    where
        L: IntoIterator<Item = Box<dyn Layout>>
    {
        self.inner.layouts = layouts.into_iter().collect();
        self
    }

    /// Sets which window classes to not be placed under layout.
    pub fn float_classes<F>(mut self, float_classes: F) -> Self
    where
        F: IntoIterator<Item = String>
    {
        self.inner.float_classes = float_classes.into_iter().collect();
        self
    }

    /// Sets the width of the gap between windows, if any.
    pub fn gap_px(mut self, gap_px: u32) -> Self {
        self.inner.gap_px = gap_px;
        self
    }

    /// Sets the main ratio increment.
    pub fn main_ratio_inc(mut self, main_ratio_inc: f64) -> Self {
        self.inner.main_ratio_inc = main_ratio_inc;
        self
    }

    /// Sets the border thickness, in pixels.
    pub fn border_px(mut self, border_px: u32) -> Self {
        self.inner.border_px = border_px;
        self
    }

    /// Sets the border color of unfocused windows.
    pub fn unfocused(mut self, unfocused: Color) -> Self {
        self.inner.unfocused = unfocused;
        self
    }

    /// Sets the border color of focused windows.
    pub fn focused(mut self, focused: Color) -> Self {
        self.inner.focused = focused;
        self
    }

    /// Sets the border color of urgent windows.
    pub fn urgent(mut self, urgent: Color) -> Self {
        self.inner.urgent = urgent;
        self
    }

    /// Finishes Config construction, validates it and returns
    /// a completed config if validation is successful.
    pub fn finish(self) -> Result<Config> {
        let config = self.inner;
        config.validate()?;
        Ok(config)
    }

}

//todo: add validation, builder, etc
