//! Traits and types for defining and generating window layouts.
//!
//! The core of this module is the [`Layout`] trait, you should
//! read its documentation before looking at anything else.

use std::collections::HashSet;

use tracing::debug;

use crate::core::{Ring, Screen, Workspace};
use crate::manager::RuntimeConfig;
use crate::types::Geometry;
use crate::x::XWindowID;
use crate::{Result, ToaruError, XConn};

/// A simple no-frills floating layout.
pub mod floating;
/// A simple manually-tiled layout.
pub mod tiled;
/// Types to be used to update layouts.
pub mod update;

#[doc(inline)]
pub use floating::Floating;
#[doc(inline)]
pub use tiled::DynamicTiled;

use update::Update;

/// A trait for implementing layouts.
///
/// A `Layout` is an object that enforces the layout of the windows
/// that are managed by the workspace.
///
/// # Components and Usage
///
/// A layout has two main components: A name and a policy. The name is
/// self-explanatory, and the policy is simply how the windows are
/// arranged on the screen. This policy is implemented by the
/// `layout` method, which is called by the managing workspace.
/// This method returns a set of `LayoutActions` which the workspace
/// must carry out in order to enforce the policy.
///
/// Layouts can also receive [`Update`]s, which tell the layout to modify
/// its behaviour or policy in some way. Since not all updates are
/// applicable to a particular layout (e.g. an update to shrink the
/// main window is not applicable to a floating layout), Layouts are
/// not required to comply with all updates.
///
/// # Layout Types and Styles
///
/// Layouts are generally organized into two main styles: Floating
/// and Tiled, where floating layouts have their windows free floating
/// and and have little, if any, enforcement over window positioning.
/// Tiled layouts, on the other hand, enforce window positioning in
/// various ways.
///
/// A type implementing `Layout` reports its style through the `style`
/// method in the `Layout` trait.
///
/// # Usage by a `WindowManager`
///
/// Layouts will usually be used as a trait object by the window manager.
/// Since trait objects cannot be based on `Clone`, `Layout` requires
/// a `boxed` method that clones the object as needed.
pub trait Layout: Send {
    /// The name of the Layout, used to display in some kind of status bar.
    fn name(&self) -> &str;

    /// Return the style of the layout.
    fn style(&self) -> LayoutType;

    /// Generates the actions to be taken to lay out the windows.
    ///
    /// A `LayoutCtxt` is provided to give the layout any additional
    /// information it might need to enforce its policy.
    fn layout(&self, ctxt: LayoutCtxt<'_>) -> Vec<LayoutAction>;

    /// Returns a boxed version of itself, so it can be used a trait object.
    fn boxed(&self) -> Box<dyn Layout>;

    /// Receive an update to modify its current settings.
    /// This type does not need to respond to all possible updates,
    /// only the ones that specifically apply to it.
    fn receive_update(&self, update: &Update);
}

use custom_debug_derive::Debug;
/// The context providing any information that the layout may need
/// to enforce its layout policy.
#[non_exhaustive]
#[derive(Debug)]
pub struct LayoutCtxt<'wm> {
    //fixme: custom debug is just a bodge rn
    /// A Connection to the X server to make queries if needed.
    #[debug(skip)]
    pub conn: &'wm dyn XConn,
    /// The runtime configuration of the window manager.
    #[debug(skip)]
    pub config: &'wm dyn RuntimeConfig,
    /// The workspace that called the Layout.
    pub workspace: &'wm Workspace,
    /// The current screen the workspace is on.
    pub screen: &'wm Screen,
}

/// A Ring of layouts applied on a workspace.
///
/// A set of layouts that a workspace can use to apply on its
/// managed windows.
///
/// ## A Note to Programmers
///
/// `Layouts` has some unique invariants that normal
/// `Rings` do not have:
///
/// 1. It must _never_ be empty.
/// 2. It must _always_ have something in focus.
/// 3. There must be _no_ name conflicts
/// (i.e. no two layouts can have the same name).
///
/// To this end, there are runtime checks on startup
/// and initialization to ensure that these invariants
/// are upheld at the start of runtime. However, these
/// checks are not always carried out in normal operation,
/// which means if they are violated in some way at this
/// point, your code may panic!
pub type Layouts = Ring<Box<dyn Layout>>;

impl Layouts {
    /// Returns Self with the given layouts and
    /// the focused item set to the first item in the Ring.
    ///
    /// Use this over `Ring::new` as it ensures that
    /// the invariants on Layouts are upheld.
    ///
    /// # Panics
    ///
    /// This method panics if any of the invariants are not upheld.
    pub fn with_layouts_validated<I>(layouts: I) -> Result<Self>
    where
        I: IntoIterator<Item = Box<dyn Layout>>,
    {
        let ret = unsafe { Self::with_layouts_unchecked(layouts) };

        ret.validate()?;

        Ok(ret)
    }

    /// Returns Self with the given layouts and the focused
    /// item set to the first item in the Ring.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that invariants 1 and 3 are upheld.
    pub unsafe fn with_layouts_unchecked<I>(layouts: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn Layout>>,
    {
        let mut ret = Ring::new();
        layouts.into_iter().for_each(|l| ret.append(l));
        ret.set_focused(0);
        ret
    }

    /// Validates the namespace and ensures there are no name conflicts.
    #[allow(clippy::len_zero)]
    pub fn validate(&self) -> Result<()> {
        if self.focused.is_none() {
            return Err(ToaruError::OtherError("no focused item".into()));
        }
        /* this is equivalent to calling self.is_empty(),
        but this more clearly expresses the invariant
        that we are trying to enforce. */
        if self.len() < 1 {
            return Err(ToaruError::OtherError("layouts is empty".into()));
        }

        let set: HashSet<&str> = self.iter().map(|l| l.name()).collect();

        if set.len() == self.len() {
            Ok(())
        } else {
            let mut all: Vec<&str> = self.iter().map(|l| l.name()).collect();
            let uniques: Vec<&str> = set.into_iter().collect();

            debug_assert!(uniques.len() < self.len());

            uniques
                .into_iter()
                .for_each(|s1| all.retain(|s2| s1 != *s2));

            Err(ToaruError::LayoutConflict(all.join(", ")))
        }
    }

    /// Generates the layout for the currently focused layout.
    pub fn gen_layout<X, C>(
        &self,
        conn: &X,
        ws: &Workspace,
        scr: &Screen,
        cfg: &C,
    ) -> Vec<LayoutAction>
    where
        X: XConn,
        C: RuntimeConfig,
    {
        debug!("self.focused is {:?}", self.focused);
        debug_assert!(self.focused().is_some(), "no focused layout");
        self.focused()
            .expect("focused layout should not be none")
            .layout(LayoutCtxt {
                workspace: ws,
                config: cfg,
                conn,
                screen: scr,
            })
    }

    /// Sends an update to the current layout.
    pub fn send_update(&self, update: Update) {
        self.focused().unwrap().receive_update(&update)
    }

    /// Sends an update to every layout within.
    pub fn broadcast_update(&self, update: Update) {
        self.iter().for_each(|ly| ly.receive_update(&update))
    }
}

impl Default for Layouts {
    /// Returns a Layout instance containing a single
    /// [`Floating`] layout.
    fn default() -> Self {
        let mut ret = Ring::new();
        ret.append(Box::new(Floating {}) as Box<dyn Layout>);

        ret
    }
}

/// The type of layout being used.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LayoutType {
    /// A simple floating layout style that
    /// does not enforce any rules.
    Floating,
    /// A layout style that controls windows to hold to a certain
    /// layout, such as dynamic or manual tiling.
    Tiled,
}

#[allow(missing_docs)]
impl LayoutType {
    pub fn is_floating(&self) -> bool {
        matches!(self, Self::Floating)
    }

    pub fn is_tiled(&self) -> bool {
        matches!(self, Self::Tiled)
    }
}

/// An action to resize a window in order to enforce the
/// layout currently in effect.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutAction {
    /// Resize a given client.
    Resize {
        /// The Client to apply the geometry to.
        id: XWindowID,
        /// The geometry to apply to the Client.
        geom: Geometry,
    },
    /// Map the given window.
    Map(XWindowID),
    /// Unmap the given window.
    Unmap(XWindowID),
    /// Stack the given window on top.
    StackOnTop(XWindowID),
    /// Remove the given window from the layout.
    Remove(XWindowID),
}
