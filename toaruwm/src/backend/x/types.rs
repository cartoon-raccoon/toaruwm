//! Types specific to the X Window Server.

use std::ops::Deref;

use tracing::error;

use crate::core::types::{Geometry, Color};

use super::{
    core::{StackMode, XAtom, XConn},
    property::{WindowState, WmHints, WmSizeHints},
};

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