//! Types for Output configuration.
//! 
//! This module provides the [`Output`] and [`OutputLayout`] types,
//! which allow you to specify how you want your monitors to be laid out in the 2D coordinate space.

use std::sync::{Arc, Weak};
use std::cell::Cell;

use strum::EnumIs;

use crate::types::{Point, Size, Physical, Logical, Cardinal, Transform};
use crate::wayland::WaylandOutput;

/// A set of outputs laid out on a 2D coordinate space, as defined by the user.
/// You can insert and remove Outputs as needed.
/// 
/// ## Insertion
/// 
/// A successful insertion into the `OutputLayout` will yield an identifier that you
/// can then use to directly address the inserted `Output`.
/// 
/// ## Matching
/// 
/// Any `Output` stored in the `OutputLayout` can be matched with a type implementing
/// [`PlatformOutput`]. This is used at runtime, to match the actual monitors that are
/// plugged in, with the ones specified in the `OutputLayout`, so they can be
/// configured accordingly.
/// 
/// There can be multiple `Outputs` in the `OutputLayout` that can match with a given
/// `PlatformOutput`, so, by default, a `PlatformOutput` is matched with the [strongest matching][1]
/// `Output`.
/// 
/// When an `Output` is matched with `PlatformOutput`, it is marked internally as such,
/// so subsequent calls to `match*` methods with `PlatformOutput`s that could potentially
/// match on that `Output` will no longer match.
/// 
/// ## Runtime
/// 
/// At runtime, the Platform implementation will get information about the actual
/// monitors you have plugged in, and try to match them with the ones you have
/// suuplied. Any matches will be configured accordingly, and any non-matches will be ignored.
/// If there are monitors that did not match any of the ones given here, a sensible default
/// configuration will be applied to them.
/// 
/// [1]: OutputIdentifier::match_with
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
                id: 0,
                matched: Cell::new(false),
            })]
        }
    }

    /// Insert an output at a specified Point.
    pub fn insert_at_point(&mut self, point: Point<i32, Logical>, output: Output) -> Result<usize, Output> {
        if let Some(oname) = output.name() {
            if self.find_by_name(oname).is_some() {
                return Err(output)
            }
        }

        let id = self.outputs.len();

        let entry = Arc::new(OutputEntry {
            inner: output,
            pos: OutputPosition::Point(point),
            id,
            matched: Cell::new(false),
        });

        self.outputs.push(entry);
        Ok(id)
    }

    /// Inserts `output` automatically into the Layout.
    pub fn insert(&mut self, output: Output) -> Result<usize, Output> {
        if let Some(oname) = output.name() {
            if self.find_by_name(oname).is_some() {
                return Err(output)
            }
        }

        todo!()
    }

    /// Insert an output into the Layout with respect to another already-inserted output.
    pub fn insert_relative_to<S: AsRef<str>>(&mut self, name: S, card: Cardinal, output: Output) -> Result<usize, Output> {
        if let Some(oname) = output.name() {
            if self.find_by_name(oname).is_some() {
                return Err(output)
            }
        }

        // get a weak pointer to the referent output
        let Some(referent) = self.entry_by_name(name).map(|entry| Arc::downgrade(entry)) 
            else { return Err(output) };

        let id = self.outputs.len();

        let entry = OutputEntry {
            inner: output,
            pos: OutputPosition::Relative(card, referent),
            id,
            matched: Cell::new(false)
        };

        self.outputs.push(Arc::new(entry));

        Ok(id)
    }

    /// Insert `output` as a mirror of the output with `name`.
    pub fn insert_mirror<S: AsRef<str>>(&mut self, name: S, output: Output) -> Result<(), Output> {
        if let Some(oname) = output.name() {
            if self.find_by_name(oname).is_some() {
                return Err(output)
            }
        }
        
        let Some(referent) = self.entry_by_name(name).map(|entry| Arc::downgrade(entry))
            else { return Err(output) };

        let entry = OutputEntry {
            inner: output,
            pos: OutputPosition::Mirroring(referent),
            id: self.outputs.len(),
            matched: Cell::new(false),
        };

        self.outputs.push(Arc::new(entry));
        Ok(())
    }

    /// Removes an output from the layout by its ID. If any other output
    /// references this output in some way, that reference is now invalidated.
    /// 
    /// If no such output exists, None is returned.
    pub fn remove(&mut self, id: usize) -> Option<Output> {
        let idx = self.outputs.iter()
            .enumerate()
            .find(|(_, output)| output.id == id)
            .map(|(idx, _)| idx)?;

        let entry = Arc::into_inner(self.outputs.remove(idx))?;
        Some(entry.into_output())
    }
    
    /// Returns a reference to the Output with `name`.
    pub fn find_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Output> {
        self.entry_by_name(name).map(|entry| entry.inner())
    }

    /// Returns a reference to the first Output that matches the provided `info`.
    pub fn find_by_info(&self, info: &OutputInfo) -> Option<&Output> {
        todo!()
    }

    /// Find the output with the strongest match for the given `PlatformOutput`.
    /// 
    /// Once an `Output` is matched, it will no longer match on subsequent
    /// calls to `match_with`, unless unmatched with [`OutputLayout::unmatch`].
    pub fn match_with(&self, output: &WaylandOutput) -> Option<&Output> {
        self.match_with_entry(output).map(|entry| entry.inner())
    }

    /// Unmatch a matched output with the given `id`.
    /// 
    /// If no output exists with the given ID, `None` is returned.
    pub fn unmatch(&self, id: usize) -> Option<&Output> {
        self.entry_by_id(id)
            .map(|entry| {
                entry.matched.set(false);
                entry.inner()
            })
    }

    pub(crate) fn entry_by_id(&self, id: usize) -> Option<&Arc<OutputEntry>> {
        self.outputs.iter()
            .find(|entry| entry.id == id)
    }

    pub(crate) fn entry_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Arc<OutputEntry>> {
        self.outputs.iter()
            .find(|entry| entry.name() == Some(name.as_ref()))
    }

    pub(crate) fn match_with_entry(&self, output: &WaylandOutput) -> Option<&Arc<OutputEntry>> {
        // get matches for all output entries
        // return the strongest match
        self.outputs.iter()
            // filter out already-matched entries
            .filter(|entry| !entry.matched.get())
            // for each entry, return a tuple containing its id and its match
            .map(|entry| (entry.id, entry.inner().identifier.match_with(output)))
            // filter out the non-matches
            .filter(|(_, maybe_match)| maybe_match.is_some())
            // unwrap the matches
            .map(|(id, maybe_match)| (id, maybe_match.unwrap()))
            // get the maximum match
            .max_by_key(|(_, maybe_match)| *maybe_match)
            // grab the entry by id
            .and_then(|(id, _)| self.entry_by_id(id))
            // set the entry as matched
            .inspect(|entry| entry.matched.set(true))
    }
}

/// A platform-agnostic representation of the configuration of a physical monitor, as managed by Toaru.
#[derive(Debug, Clone)]
pub struct Output {
    /// The identifier of the Output.
    pub identifier: OutputIdentifier,
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
    pub fn new(ident: OutputIdentifier) -> Self {
        Self {
            identifier: ident,
            mode: None,
            enabled: true,
            scale: None,
            transform: Default::default(),
            vrr: false,
        }
    }

    /// Gets the name of the Output, if any.
    pub fn name(&self) -> Option<&str> {
        match &self.identifier {
            OutputIdentifier::Name(name) | OutputIdentifier::Both { name, ..} => {
                Some(name)
            }
            OutputIdentifier::Info(_) => None
        }
    }
}

/// An identifier for an `Output` that can be matched on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputIdentifier {
    /// An output identified by name only, usually formatted `<connector>-<number>` (e.g. "eDP-1").
    Name(String),
    /// An output identified by its make and model only.
    Info(OutputInfo),
    /// An output identified by both.
    #[allow(missing_docs)]
    Both{
        name: String,
        info: OutputInfo
    }
}

impl OutputIdentifier {
    /// Attempt to match the `Identifier` with a `PlatformOutput`.
    /// 
    /// The return value corresponds to the strength of the match. The more fields
    /// match, the stronger the match.
    /// 
    /// If there is no match, `None` is returned.
    pub fn match_with(&self, output: &WaylandOutput) -> Option<usize> {
        match self {
            OutputIdentifier::Name(name) => {
                if name == &output.name() {
                    Some(1)
                } else {
                    None
                }
            }
            OutputIdentifier::Info(info) => {
                let OutputInfo { make, model } = info;

                // there should be at least one field filled out
                if make.is_none() && model.is_none() {
                    return None
                }

                let output_info = output.info();

                let mut ret = 0;

                if let Some(make) = make {
                    if let Some(o_make) = &output_info.make {
                        if make == o_make {
                            ret += 1;
                        } else {
                            return None
                        }
                    }
                }
                if let Some(model) = model {
                    if let Some(o_model) = &output_info.model {
                        if model == o_model {
                            ret += 1;
                        } else {
                            return None
                        }
                    }
                }

                Some(ret)
            }
            OutputIdentifier::Both{ name, info } => {
                let name_match = OutputIdentifier::Name(name.clone()).match_with(output);
                let info_match = OutputIdentifier::Info(info.clone()).match_with(output);

                // if either one does not match, reject the entire thing
                if name_match.is_none() || info_match.is_none() {
                    return None
                }

                Some(name_match.unwrap() + info_match.unwrap())
            }
        }
    }
}

/// Any additional info about an output.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputInfo {
    /// The make of the output, usually the brand of the monitor.
    pub make: Option<String>,
    /// The model of the output.
    pub model: Option<String>,
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
    pub(crate) id: usize,
    pub(crate) matched: Cell<bool>
}

impl OutputEntry {
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
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

#[cfg(test)]
mod test {

}