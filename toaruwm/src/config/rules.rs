//! Window Rules for use in a config.

use std::hash::{Hash, Hasher};
use std::mem;

use indexmap::set::IndexSet;

/// A rule that can be applied to windows.
#[derive(Debug, Clone)]
pub struct WindowRule {
    // fixme: keep IndexSet for now; if order doesn't matter switch to HashSet
    pub(crate) directives: IndexSet<Directive>,
}

impl WindowRule {
    /// Creates an empty WindowRule.
    pub fn empty() -> Self {
        Self {
            directives: IndexSet::new(),
        }
    }

    /// Creates a WindowRule with the given directives.
    pub fn new<D>(directives: D) -> Self
    where
        D: IntoIterator<Item = Directive>
    {
        Self {
            directives: directives.into_iter().collect()
        }
    }

    /// Inserts a new directive into the window rule.
    pub fn insert_directive(&mut self, directive: Directive) {
        self.directives.insert(directive);
    }
}

/// Directives to control what the WindowRule matches on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Directive {
    /// Match on this parameter.
    Match(Parameter),
    /// Exclude anything that matches this parameter (the complement of Match).
    Exclude(Parameter),
}

/// Parameters that can be matched on.
/// 
/// `Parameter` implements `PartialEq` and `Eq`
/// such that if two instances are the same variant, they
/// will be equal, regardless of the contained value.
/// For example:
/// 
/// ```
/// use toaruwm::config::rules::Parameter;
/// 
/// let lhs = Parameter::Maximized(false);
/// let rhs = Parameter::Maximized(true);
/// 
/// assert_eq!(lhs, rhs);
/// ```
#[derive(Debug, Clone)]
pub enum Parameter {
    /// The current title of the window.
    Title(String),
    /// The current `app_id` of the window.
    AppId(String),
    /// The title of the window when it was first created.
    InitialTitle(String),
    /// The `app_id` of the window when it was first created.
    InitialAppId(String),
    /// Whether the window is floating.
    Floating(bool),
    /// Whether the window is currently fullscreened.
    Fullscreen(bool),
    /// Whether the window is currently maximized.
    Maximized(bool),
    /// Whether the window is currently pinned.
    Pinned(bool),
    /// Whether the window is the currently focused window.
    Focused(bool),
    /// Whether the window is grouped.
    Grouped(bool),
    /// The fullscreen state of the window.
    FullscreenState {
        /// Whether the server is tracking the window as fullscreen.
        internal: bool, 
        /// Whether the client knows it is fullscreen.
        client: bool 
    },
}

impl PartialEq for Parameter {
    fn eq(&self, rhs: &Parameter) -> bool {
        mem::discriminant(self) == mem::discriminant(rhs)
    }
}

impl Eq for Parameter {}

// implement Hash to hash the enum's discriminant, ignoring the contained value.
impl Hash for Parameter {
    fn hash<H>(&self, h: &mut H) where H: Hasher {
        mem::discriminant(self).hash(h);
    }
}