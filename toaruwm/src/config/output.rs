//! Types for Output configuration.

use std::sync::{Arc, Weak};

use strum::EnumIs;

use crate::types::{Point, Size, Physical, Logical, Cardinal, Transform};

/// A set of outputs laid out on a 2D coordinate space, as defined by the user.
/// You can insert and remove Outputs as needed.
/// 
/// At runtime, the Platform implementation will get information about the actual
/// monitors you have plugged in. Any monitors specified in the Layout that cannot
/// be found on your system will be removed, and the remaining monitors will be
/// repositioned best to fit.
#[derive(Debug, Default)]
pub struct OutputLayout {
    pub(crate) outputs: Vec<Arc<OutputEntry>>
}

impl OutputLayout {
    /// Creates a new OutputLayout.
    pub fn new() -> Self {
        Self {
            outputs: Vec::new()
        }
    }

    /// Checks if the OutputLayout is empty.
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    /// Returns the number of Outputs in the OutputLayout.
    pub fn size(&self) -> usize {
        self.outputs.len()
    }

    /// Creates a new OutputLayout with the provided output.
    pub fn with_output(output: Output) -> Self {
        Self {
            outputs: vec![Arc::new(OutputEntry {
                inner: output, 
                pos: OutputPosition::Point(Point::zeroed()),
                //idx: 0
            })]
        }
    }

    /// Insert an output at a specified Point.
    pub fn insert_at_point(&mut self, point: Point<i32, Logical>, output: Output) -> Result<(), Output> {
        if self.find_by_name(&output.name).is_some() {
            return Err(output)
        }

        let entry = Arc::new(OutputEntry {
            inner: output,
            pos: OutputPosition::Point(point),
        });

        self.outputs.push(entry);
        Ok(())
    }

    /// Inserts `output` automatically into the Layout.
    pub fn insert(&mut self, output: Output) -> Result<(), Output> {
        if self.find_by_name(&output.name).is_some() {
            return Err(output)
        }

        todo!()
    }

    /// Insert an output into the Layout with respect to another already-inserted output.
    pub fn insert_relative_to<S: AsRef<str>>(&mut self, name: S, card: Cardinal, output: Output) -> Result<(), Output> {
        if self.find_by_name(&output.name).is_some() {
            return Err(output)
        }

        // get a weak pointer to the referent output
        let Some(referent) = self.entry_by_name(name).map(|entry| Arc::downgrade(entry)) 
            else { return Err(output) };


        let entry = OutputEntry {
            inner: output,
            pos: OutputPosition::Relative(card, referent),
        };

        self.outputs.push(Arc::new(entry));

        Ok(())
    }

    /// Insert `output` as a mirror of the output with `name`.
    pub fn insert_mirror<S: AsRef<str>>(&mut self, name: S, output: Output) -> Result<(), Output> {
        if self.find_by_name(&output.name).is_some() {
            return Err(output)
        }
        
        let Some(referent) = self.entry_by_name(name).map(|entry| Arc::downgrade(entry))
            else { return Err(output) };

        let entry = OutputEntry {
            inner: output,
            pos: OutputPosition::Mirroring(referent),
        };

        self.outputs.push(Arc::new(entry));
        Ok(())
    }

    /// Removes an output from the layout by name. If any other output
    /// references this output in some way, that reference is now invalidated.
    /// 
    /// If no such output exists, None is returned.
    pub fn remove<S: AsRef<str>>(&mut self, name: S) -> Option<Output> {
        let idx = self.outputs.iter()
            .enumerate()
            .find(|(_, output)| output.name() == name.as_ref())
            .map(|(idx, _)| idx)?;

        let entry = Arc::into_inner(self.outputs.remove(idx))?;
        Some(entry.into_output())
    }

    /// Returns a reference to the Output with `name`.
    pub fn find_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Output> {
        self.entry_by_name(name).map(|entry| entry.inner())
    }

    pub(crate) fn entry_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Arc<OutputEntry>> {
        self.outputs.iter()
            .find(|entry| entry.name() == name.as_ref())
    }
}

/// A platform-agnostic representation of a physical monitor, as managed by Toaru.
#[derive(Debug, Clone)]
pub struct Output {
    /// The name of the output, usually formatted `<connector>-<number>` (e.g. "eDP-2").
    pub name: String,
    /// Whether the output should be enabled.
    pub enabled: bool,
    /// Mode of the output. If none, preferred will be chosen.
    pub mode: Option<OutputMode>,
    /// Scale used by the output. If none, preferred will be chosen.
    pub scale: Option<OutputScale>,
    /// The transform to be used by the Output.
    pub transform: Transform,
    /// Whether variable refresh-rate is enabled for this output.
    pub vrr: bool,
}

impl Output {
    /// Creates a new output with defaults.
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
            mode: None,
            enabled: true,
            scale: None,
            transform: Default::default(),
            vrr: false,
        }
    }
}

/// A platform, agnostic representation of a physical monitor's mode, as managed by Toaru.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutputMode {
    /// The resolution of the mode.
    pub size: Size<i32, Physical>,
    /// The refresh rate of the mode.
    pub refresh: i32,
}

/// The scale used by an Output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputScale {
    /// An integer scale.
    Integer(i32),
    /// A fractional scale, used when supporting fractional scaling.
    Fractional(f64),
    /// A scale split between the values advertised to clients.
    Split {
        /// An integer advertised to clients that do not support fractional scaling.
        integer: i32,
        /// A fractional scale value used elsewhere.
        fractional: f64
    }
}

/// An entry in an OutputLayout.
#[derive(Debug, Clone)]
pub(crate) struct OutputEntry {
    pub(crate) inner: Output,
    pub(crate) pos: OutputPosition,
}

impl OutputEntry {
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn inner(&self) -> &Output {
        &self.inner
    }

    pub fn into_output(self) -> Output {
        self.inner
    }
}

/// The (intended) position of an output in the layout.
#[derive(Debug, Clone, EnumIs)]
pub(crate) enum OutputPosition {
    /// At a requested point on the global coordinate space.
    Point(Point<i32, Logical>),
    /// Relative to another Output.
    Relative(Cardinal, Weak<OutputEntry>),
    /// Mirroring another Output.
    Mirroring(Weak<OutputEntry>)
}