//! Types for configuring a `WindowManager`.
//!
//! This module contains `Config`, the trait that defines a
//! configuration object usable by a `WindowManager`. One type
//! that implements this is already provided: [`ToaruConfig`],
//! and you can this directly in a `WindowManager`.
//!
use custom_debug_derive::Debug;

use std::any::Any;
use std::collections::HashMap;

use crate::core::WorkspaceSpec;
use crate::layouts::{
    DynamicTiled, Floating, Layout,
    update::{IntoUpdate, UpdateBorderPx},
};
use crate::manager::state::{RuntimeConfig, WmConfig};
use crate::types::Color;
use crate::{Result, ToaruError::*};

/// A trait defining a `WindowManager` configuration.
///
/// On initialization, the `WindowManager` queries a config
/// for various fields to move elsewhere, before at the end converting
/// the config into a runtime configuration.
///
/// You will probably have noticed that the `workspaces` and
/// `layouts` methods take a `&mut self`, but return owned
/// types. This is because any type implementing this trait
/// is expected to be dropped by the end of the window manager
/// initialization process, where it will be turned into a
/// type implementing `RuntimeConfig`. Thus, it can afford
/// to do some expensive operations such as cloning. These methods
/// also use a `&mut self` to allow the user to mutate internal
/// state, or to do something like [`std::mem::take`].
///
/// The type implementing this trait must yield these key fields
/// for the window manager to take and initialize
/// itself, before converting itself into a `RuntimeConfig`.
///
/// # Configuration Keys
///
/// The required configuration keys are:
///
/// - *Layouts*: the set of layouts used by the window manager.
/// - *Workspaces*: the workspaces to be created by the window
/// manager.
///
/// The following are not explicitly required by `Config`, but
/// are required by the [`RuntimeConfig`] trait, which `Self::Config`
/// needs to implement:
///
/// - *Float Classes*: the set of window classes that the window
/// manager will not place under layout.
/// - *Border Pixel*: The thickness of the window border.
/// - *Unfocused*: The border color of unfocused windows.
/// - *Focused*: The border color of focused windows.
/// - *Urgent*: The border color of focused windows.
///
/// `RuntimeConfig` also requires a method `get_key` to retrieve arbitrary
/// values of keys.
///
/// # Validity
///
/// While user-defined keys may have their own invariants that
/// should not violated, A type `Config` also has one invariant of its own,
/// that its Layouts and Workspaces must contain at least one member,
/// i.e. they cannot be empty.
pub trait Config {
    /// The type it will finally convert itself into.
    type Runtime: RuntimeConfig;

    /// The workspace collection returned when queried.
    type Workspaces: IntoIterator<Item = WorkspaceSpec>;

    /// The layout collection returned when queried.
    type Layouts: IntoIterator<Item = Box<dyn Layout>>;

    /// Yield an iterator over the workspaces.
    fn take_workspaces(&mut self) -> Self::Workspaces;

    /// Yield an iterator over the layouts.
    fn take_layouts(&mut self) -> Self::Layouts;

    /// Perform the conversion into the RuntimeConfig.
    fn into_runtime_config(self) -> Self::Runtime;
}

/// The central configuration object.
///
/// `ToaruConfig` stores several key attributes that are required
/// by the window manager to run, but it can also store
/// any arbitrary key-value pair.
///
///
/// `ToaruConfig` provides a `validate` method that ensures it is valid
/// and can be used in a `WindowManager`. While this checks the
/// predefined invariants on the Config, it can also run user-defined
/// code to ensure that user-defined invariants are also upheld.
///
/// # Construction
///
/// To build a ToaruConfig, use the [`ToaruConfigBuilder`] type.
///
/// # Example
///
/// ```rust
/// # use toaruwm::manager::config::NO_CHECKS;
/// use toaruwm::ToaruConfig;
///
/// // create a default config that upholds all invariants
/// let config = ToaruConfig::new();
///
/// config.validate(NO_CHECKS).expect("invalid config");
/// ```
#[derive(Debug)] //todo: impl Clone, Debug
pub struct ToaruConfig {
    /// The workspaces and the screen it should be sent to.
    pub(crate) workspaces: Vec<WorkspaceSpec>,
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
    /// Storage for any user-defined keys.
    pub(crate) keys: HashMap<String, Box<dyn Any>>,
}

//* I would use an Option<F> instead of doing this bodge, but
//* passing in None would cause type inference issues.
const fn no_checks(_: &ToaruConfig) -> Result<()> {
    Ok(())
}

/// A constant signifying no user-defined checks are required.
///
/// Pass this into `ToaruConfig::validate` if you have no additional
/// validation checks to run on your config.
pub const NO_CHECKS: fn(&ToaruConfig) -> Result<()> = no_checks;

impl ToaruConfig {
    /// Returns the default construction.
    pub fn new() -> Self {
        let ret = Self::default();
        ret.validate(NO_CHECKS).unwrap();
        ret
    }

    /// Returns a [`ToaruConfigBuilder`] to build your Config with the
    /// 'builder' idiom.
    pub fn builder() -> ToaruConfigBuilder {
        ToaruConfigBuilder::new()
    }

    /// Checks the configuration to verify that all invariants are upheld.
    ///
    /// This is useful if your `ToaruConfig` goes through a bunch of
    /// additional processing before it's ready to use in a `WindowManager`,
    /// and you want to make sure that all the invariants that you
    /// need it to uphold are indeed upheld. To help with this, you can insert
    /// additional code to check that your user-added keys are valid.
    ///
    /// If you have no code you want to insert, pass in the
    /// [`NO_CHECKS`] constant.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::config::{ToaruConfig, NO_CHECKS};
    /// use toaruwm::{Result, ToaruError::*};
    ///
    /// let mut config = ToaruConfig::new();
    ///
    /// // insert a user-defined key into the Config
    /// // that requires us to validate
    /// config.insert_key("foo", 1i32);
    ///
    /// // run the validation
    /// config.validate(|cfg: &ToaruConfig| {
    ///     let foo = cfg.get_key::<i32>("foo");
    ///     if let Some(_) = foo {
    ///         Ok(())
    ///     } else {
    ///         Err(InvalidConfig("missing foo".into()))
    ///     }
    /// }).expect("config was invalid!");
    ///
    /// // now, let's create a mew config that doesn't require
    /// // and user-defined validation
    /// let config2 = ToaruConfig::new();
    ///
    /// config2.validate(NO_CHECKS).expect("invalid config2");
    /// ```
    #[allow(clippy::len_zero)]
    pub fn validate<F>(&self, checks: F) -> Result<()>
    where
        F: FnOnce(&ToaruConfig) -> Result<()>,
    {
        if self.workspaces.len() < 1 {
            return Err(InvalidConfig("workspaces is empty".into()));
        }
        if self.layouts.len() < 1 {
            return Err(InvalidConfig("layouts is empty".into()));
        }
        checks(self)?;
        Ok(())
    }

    /// Inserts an arbitrary key-value pair into the Config.
    pub fn insert_key<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Any,
    {
        self.keys.insert(key.into(), Box::new(value) as Box<V>);
    }

    /// Remove a key-value pair from the Config.
    ///
    /// Returns None if the value doesn't exist or is not of
    /// the specified type.
    ///
    /// Since this function is so generic, it is likely
    /// you will often have to use Rust's 'turbofish'
    /// notation (`::<T>`) to specify the type of the value
    /// you want to retrieve.
    pub fn remove_key<V: Any>(&mut self, key: &str) -> Option<V> {
        self.keys
            .remove(&String::from(key))
            .and_then(|v| v.downcast().ok())
            .map(|v| *v)
    }

    /// Introspection into the workspaces set on the Config.
    pub fn workspaces(&self) -> &[WorkspaceSpec] {
        &self.workspaces
    }

    /// All layouts available to the windowmanager to use.
    pub fn layouts(&self) -> &[Box<dyn Layout>] {
        &self.layouts
    }

    /// All the window classes that should not be set under layout.
    pub fn float_classes(&self) -> &[String] {
        &self.float_classes
    }

    /// The thickness of the window borders, in pixels.
    pub fn border_px(&self) -> u32 {
        self.border_px
    }

    /// The border color of unfocused windows.
    pub fn unfocused(&self) -> Color {
        self.unfocused
    }

    /// The border color of focused windows.
    pub fn focused(&self) -> Color {
        self.focused
    }

    /// The border colour of urgent windows.
    pub fn urgent(&self) -> Color {
        self.urgent
    }

    /// Get a generic key from the `Config`'s internal store.
    ///
    /// Returns `None` if the key does not exist or is not
    /// in the type specified.
    ///
    /// Since this function is so generic, it is likely
    /// you will often have to use Rust's 'turbofish'
    /// notation (`::<T>`) to specify the type of the value
    /// you want to retrieve.
    pub fn get_key<V: Any>(&self, key: &str) -> Option<&V> {
        self.keys
            .get(&String::from(key))
            .and_then(|i| i.downcast_ref::<V>())
    }
}

impl Config for ToaruConfig {
    type Runtime = WmConfig;
    type Workspaces = Vec<WorkspaceSpec>;
    type Layouts = Vec<Box<dyn Layout>>;

    fn take_workspaces(&mut self) -> Vec<WorkspaceSpec> {
        self.workspaces.clone()
    }

    fn take_layouts(&mut self) -> Vec<Box<dyn Layout>> {
        self.layouts.iter().map(|l| l.boxed()).collect()
    }

    fn into_runtime_config(self) -> Self::Runtime {
        WmConfig {
            float_classes: self.float_classes,
            border_px: self.border_px,
            unfocused: self.unfocused,
            focused: self.focused,
            urgent: self.urgent,
            keys: self.keys,
        }
    }
}

impl Default for ToaruConfig {
    fn default() -> ToaruConfig {
        let layouts = vec![String::from("DTiled"), String::from("Floating")];
        ToaruConfig {
            workspaces: vec![
                WorkspaceSpec::new("1", 0, layouts.clone()),
                WorkspaceSpec::new("2", 0, layouts.clone()),
                WorkspaceSpec::new("3", 0, layouts),
            ],
            layouts: vec![
                Box::new(DynamicTiled::new(0.5, 2)) as Box<dyn Layout>,
                Box::new(Floating::new()) as Box<dyn Layout>,
            ],
            float_classes: Vec::new(),
            border_px: 2,
            unfocused: Color::from(0x555555),
            focused: Color::from(0xdddddd),
            urgent: Color::from(0xee0000),
            keys: {
                let mut keys = HashMap::new();
                keys.insert("gap_px".into(), Box::new(0u32) as Box<dyn Any>);
                keys.insert("main_ratio_inc".into(), Box::new(0.05f32) as Box<dyn Any>);

                keys
            },
        }
    }
}

/// A helper type to construct a [`ToaruConfig`].
//todo: add example
#[derive(Debug, Default)]
pub struct ToaruConfigBuilder {
    inner: ToaruConfig,
}

impl ToaruConfigBuilder {
    /// Creates a new `ConfigBuilder`.
    pub fn new() -> Self {
        Self {
            inner: ToaruConfig::default(),
        }
    }

    /// Sets the workspaces used by the WindowManager.
    pub fn workspaces<W>(mut self, workspaces: W) -> Self
    where
        W: IntoIterator<Item = WorkspaceSpec>,
    {
        self.inner.workspaces = workspaces.into_iter().collect();
        self
    }

    /// Sets the layouts used by the WindowManager.
    pub fn layouts<L>(mut self, layouts: L) -> Self
    where
        L: IntoIterator<Item = Box<dyn Layout>>,
    {
        self.inner.layouts = layouts.into_iter().collect();
        self
    }

    /// Sets which window classes to not be placed under layout.
    pub fn float_classes<F>(mut self, float_classes: F) -> Self
    where
        F: IntoIterator<Item = String>,
    {
        self.inner.float_classes = float_classes.into_iter().collect();
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

    /// Inserts any additional keys the user may want.
    pub fn other_key<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Any,
    {
        self.inner
            .keys
            .insert(key.into(), Box::new(value) as Box<dyn Any>);
        self
    }

    /// Finishes Config construction, validates it and returns
    /// a completed config if validation is successful.
    ///
    /// You can supply an additional `check` to run
    /// additional code to validate your config.
    pub fn finish<F>(self, check: F) -> Result<ToaruConfig>
    where
        F: FnOnce(&ToaruConfig) -> Result<()>,
    {
        let config = self.inner;
        for layout in config.layouts.iter() {
            layout.receive_update(
                &UpdateBorderPx(config.border_px).into_update()
            )
        }
        config.validate(check)?;
        Ok(config)
    }
}
