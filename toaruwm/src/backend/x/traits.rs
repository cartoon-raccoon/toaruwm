//! Traits that define the interface to the X Server.
//! 
//! 
//! # Trait Structure
//!
//! This module contains three main traits which define the
//! interface between ToaruWM and the X server:
//! 
//! - [`XCore`] which defines core protocol requests,
//! 
//! - [`RandR`], which defines compatibility with the
//!  Resize and Rotate (RandR) extension, and
//! 
//! - [`Xkb`], which defines compatibility with the X Keyboard
//! Extension. 
//! 
//! All these traits are combined by the main [`XConn`]
//! trait, which the main [`WindowManager`][1] uses:
//! 
//! ```text
//!       WindowManager
//!             |
//!             |
//!           XConn
//!             |
//!             |
//!   ----------|----------
//!   |         |         |
//!   |         |         |
//! XCore     Randr      Xkb
//! ```
//! 
//! ToaruWM makes use of RandR and XKB to position displays
//! and parse keyboard bindings respectively, so `XConn` is simply
//! a marker trait requiring the three subtraits [`XCore`], [`RandR`] and
//! [`Xkb`], which define the actual interface with the X server. For
//! more information on required methods for each of these traits,
//! consult their documentation.
//! 
//! [1]: crate::WindowManager
use std::str::FromStr;

use tracing::{debug, error, warn};

use crate::bindings::{Keybind, Mousebind};

use super::{
    core::{Xid, XWindowID, Result, XAtom, XError, PointerQueryReply, XWindow, WindowClass},
    atom::{Atom, AUTO_FLOAT_WINDOW_TYPES, UNMANAGED_WINDOW_TYPES},
    event::{ClientMessageEvent, XEvent},
    property::{Property, WmHints, WmSizeHints, WindowState},
    types::{ClientAttrs, ClientConfig, XWinProperties},
};
use crate::core::Screen;
use crate::core::types::{Geometry};

/// A trait used to define the interface between ToaruWM and the X server.
///
/// XConn provides an abstraction layer for talking to an underlying X server.
/// Its methods are designed to provide as thin a layer as possible,
/// often mapping directly to X server protocol requests, with type
/// conversion to present dependency-agnostic types.
///
/// # Usage in a WindowManager
///
/// An implementation of `XConn` is required for using a [WindowManager][1].
/// The backend library used does not directly appear inside `WindowManager`.
/// Thus, it is possible to create your own XConn type using a different
/// library, possibly using XLib, and in theory this crate can run on
/// any display server implementing the X protocol, given a proper
/// implementor of `XConn`.
/// 
/// # Implementors
///
/// This crate provides two implementations of XConn: [XCBConn][2] and
/// [X11RBConn][3].
///
/// [1]: crate::manager::WindowManager
/// [2]: crate::x::xcb::XCBConn
/// [3]: crate::x::x11rb::X11RBConn
pub trait XConn: XCore + RandR + Xkb {}

/// A trait defining the core protocol interface between ToaruWM
/// and the X server.
/// 
/// This trait contains methods that define the X server functionality
/// required by ToaruWM, such as window creation and configuration,
/// mapping and unmapping, atom management, etc. For more information
/// on the overall trait structure, see the module-level documentation.
/// 
/// # Atom Management
///
/// An XConn implementation should also provide a way to manage X atoms.
/// Its `atom()` method should intern an Atom if not known, and
/// the implementation should store this in its internal state in some way.
/// While this functionality is not required, it is heavily encouraged.
/// 
/// # XKB and Randr capability
/// 
/// While this is not specified in the required methods, implementors should
/// account for XKB and RandR events.
pub trait XCore {
    //* General X server operations

    /// Receives the next event from the X server.
    ///
    /// If no events are queued, returns Ok(None),
    /// allowing the event loop to continue and handle other processing.
    /// If the connection has an error, it returns Err.
    ///
    /// Else, it returns Ok(Some(event)).
    /// 
    /// # XKB and RandR
    /// 
    /// This method should also account for XKB and RandR
    /// events and process those accordingly.
    fn poll_next_event(&self) -> Result<Option<XEvent>>;

    /// Returns the ID and geometry of the root window.
    fn get_root(&self) -> XWindow;

    /// Returns the geometry of a given window.
    fn get_geometry(&self, window: XWindowID) -> Result<Geometry>;

    /// Queries the given window and all its children.
    fn query_tree(&self, window: XWindowID) -> Result<Vec<XWindowID>>;

    /// Queries the X server for pointer data.
    fn query_pointer(&self, window: XWindowID) -> Result<PointerQueryReply>;

    /// Returns randr data on all connected screens.
    fn all_outputs(&self) -> Result<Vec<Screen>>;

    /// Get the value of an atom by its name.
    ///
    /// You can use [Atom][1]'s `as_ref()` method to get a
    /// known atom's string representation.
    ///
    /// If the atom is unknown, intern it.
    ///
    /// [1]: crate::x::atom::Atom
    fn atom(&self, atom: &str) -> Result<XAtom>;

    /// Looks up the name of an atom.
    fn lookup_atom(&self, atom: XAtom) -> Result<String>;

    /// Looks up the value of an interned atom given its name.
    ///
    /// If the atom is not interned, None should be returned.
    fn lookup_interned_atom(&self, name: &str) -> Option<XAtom>;

    /// Grabs the keyboard.
    fn grab_keyboard(&self) -> Result<()>;

    /// Ungrabs the keyboard.
    fn ungrab_keyboard(&self) -> Result<()>;

    /// Grabs a key-modmask combo for a given window.
    fn grab_key(&self, kb: Keybind, window: XWindowID) -> Result<()>;

    /// Ungrabs a key-modmask combo for a given window.
    fn ungrab_key(&self, kb: Keybind, window: XWindowID) -> Result<()>;

    /// Grabs a mouse button-mask combo for a given window.
    ///
    /// `confine` denotes whether or not the event should be generated.
    fn grab_button(&self, mb: Mousebind, window: XWindowID, confine: bool) -> Result<()>;

    /// Ungrabs a mouse button-mask combo for a given window.
    fn ungrab_button(&self, mb: Mousebind, window: XWindowID) -> Result<()>;

    /// Grabs the pointer.
    fn grab_pointer(&self, winid: XWindowID, mask: u32) -> Result<()>;

    /// Ungrabs the pointer.
    fn ungrab_pointer(&self) -> Result<()>;

    //* Window-related operations
    /// Create a new window.
    fn create_window(&self, ty: WindowClass, geom: Geometry, managed: bool) -> Result<XWindowID>;

    /// Maps a given window.
    fn map_window(&self, window: XWindowID) -> Result<()>;

    /// Unmaps a given window.
    fn unmap_window(&self, window: XWindowID) -> Result<()>;

    /// Destroys a window.
    ///
    /// Implementors should make use of the provided
    /// `XConn::win_supports()` method to delete the window
    /// via ICCCM WM_DELETE_WINDOW if supported.
    fn destroy_window(&self, window: XWindowID) -> Result<()>;

    /// Sends a message to a given client.
    fn send_client_message(&self, window: XWindowID, data: ClientMessageEvent) -> Result<()>;

    /// Sets the input focus to a given window.
    fn set_input_focus(&self, window: XWindowID) -> Result<()>;

    /// Set the geometry for a given window.
    fn set_geometry(&self, window: XWindowID, geom: Geometry) -> Result<()>;

    /// Set the property for a given window.
    fn set_property(&self, window: XWindowID, prop: &str, data: Property) -> Result<()>;

    /// Retrieves a given property for a given window by its atom name.
    fn get_property(&self, prop: &str, window: XWindowID) -> Result<Option<Property>>;

    /// Sets the root screen.
    fn set_root_scr(&mut self, scr: i32);

    /// Change window attributes for a given window.
    fn change_window_attributes(&self, window: XWindowID, attrs: &[ClientAttrs]) -> Result<()>;

    /// Configure a given window.
    fn configure_window(&self, window: XWindowID, attrs: &[ClientConfig]) -> Result<()>;

    /// Reparent a window under a given parent.
    fn reparent_window(&self, window: XWindowID, parent: XWindowID) -> Result<()>;
    //fn create_window(&self);

    //* PROVIDED METHODS *//

    // ICCCM-related operations
    /// Gets all ICCCM-defined client properties.
    fn get_client_properties(&self, window: XWindowID) -> XWinProperties {
        XWinProperties {
            wm_name: self.get_wm_name(window),
            wm_icon_name: self.get_wm_icon_name(window),
            wm_size_hints: self.get_wm_size_hints(window),
            wm_hints: self.get_wm_hints(window),
            wm_class: self.get_wm_class(window),
            wm_protocols: self.get_wm_protocols(window),
            wm_state: self.get_wm_state(window),
        }
    }

    /// Gets _NET_WM_NAME or failing which, WM_NAME.
    ///
    /// Returns an empty string in case of error or if neither is set.
    fn get_wm_name(&self, window: XWindowID) -> String {
        let prop = self.get_property(Atom::NetWmName.as_ref(), window);

        match prop {
            Ok(prop) => {
                if let Some(prop) = prop {
                    match prop {
                        Property::String(mut s) | Property::UTF8String(mut s) => {
                            return s.remove(0)
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => error!("{}", e),
        }

        let prop = self.get_property(Atom::WmName.as_ref(), window);

        match prop {
            Ok(prop) => {
                if let Some(prop) = prop {
                    match prop {
                        Property::String(mut s) | Property::UTF8String(mut s) => {
                            return s.remove(0)
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => error!("{}", e),
        }

        String::from("")
    }

    /// Gets WM_ICON_NAME.
    ///
    /// Returns an empty string in case of error.
    fn get_wm_icon_name(&self, window: XWindowID) -> String {
        let prop = self.get_property(Atom::WmIconName.as_ref(), window);

        match prop {
            Ok(prop) => {
                if let Some(prop) = prop {
                    match prop {
                        Property::String(mut s) | Property::UTF8String(mut s) => s.remove(0),
                        _ => "".into(),
                    }
                } else {
                    "".into()
                }
            }
            Err(_) => "".into(),
        }
    }

    /// Gets WM_NORMAL_HINTS.
    ///
    /// Returns None if not set or in case of error.
    fn get_wm_size_hints(&self, window: XWindowID) -> Option<WmSizeHints> {
        let prop = self
            .get_property(Atom::WmNormalHints.as_ref(), window)
            .ok()?;

        if let Some(Property::WMSizeHints(sh)) = prop {
            Some(sh)
        } else {
            debug!("Got wrong property: {:?}", prop);
            None
        }
    }

    /// Gets WM_HINTS.
    ///
    /// Returns None if not set or in case of error.
    fn get_wm_hints(&self, window: XWindowID) -> Option<WmHints> {
        let prop = self.get_property(Atom::WmHints.as_ref(), window).ok()?;

        if let Some(Property::WMHints(hints)) = prop {
            Some(hints)
        } else {
            debug!("Got wrong property: {:?}", prop);
            None
        }
    }

    /// Checks whether the the `WM_HINTS` property has the accepts-input
    /// flag set.
    fn accepts_input(&self, window: XWindowID) -> bool {
        if let Some(hints) = self.get_wm_hints(window) {
            hints.accepts_input
        } else {
            false
        }
    }

    /// Gets WM_CLASS.
    ///
    /// Returns a tuple of empty strings if not set or in case of error.
    fn get_wm_class(&self, window: XWindowID) -> (String, String) {
        use Property::{String, UTF8String};

        let prop = self
            .get_property(Atom::WmClass.as_ref(), window)
            .unwrap_or(None);

        match prop {
            Some(String(strs)) | Some(UTF8String(strs)) => (strs[0].to_owned(), strs[1].to_owned()),
            _ => {
                debug!(target: "get_wm_class", "expected strings, got: {:?}", prop);
                ("".into(), "".into())
            }
        }
    }

    /// Gets WM_PROTOCOLS.
    ///
    /// Returns None if not set or in case of error.
    fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<XAtom>> {
        let prop = self.get_property(Atom::WmProtocols.as_ref(), window).ok()?;

        if let Some(Property::Atom(atoms)) = prop {
            let mut ret = Vec::new();
            for atom in atoms {
                ret.push(self.atom(&atom).ok()?)
            }
            Some(ret)
        } else {
            None
        }
    }

    /// Check whether a window supports the given protocol.
    fn win_supports(&self, protocol: &str, id: XWindowID) -> bool {
        self.atom(protocol)
            .map(|atom| {
                self.get_wm_protocols(id)
                    .map(|protocols| protocols.contains(&atom))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    /// Gets ICCCM's `WM_STATE` hint.
    fn get_wm_state(&self, window: XWindowID) -> Option<WindowState> {
        let prop = self.get_property(Atom::WmState.as_ref(), window).ok()?;

        if let Some(Property::U32List(s, list)) = prop {
            if s != Atom::WmState.as_ref() {
                error!("Got wrong type for wm_state: {}", s);
                return None;
            }
            Some(match list[0] as i32 {
                1 => WindowState::Normal,
                3 => WindowState::Iconic,
                0 => WindowState::Withdrawn,
                n => {
                    error!("Expected 1, 3, or 0 for WM_STATE, got {}", n);
                    return None;
                }
            })
        } else {
            debug!(target: "get_wm_state", "window {} did not set WM_STATE", window);
            None
        }
    }

    /// Gets ICCCM's `WM_TRANSIENT_FOR` hint.
    fn get_wm_transient_for(&self, window: XWindowID) -> Option<XWindowID> {
        let prop = self
            .get_property(Atom::WmTransientFor.as_ref(), window)
            .ok()?;

        if let Some(Property::Window(ids)) = prop {
            if ids[0] == Xid(0) {
                warn!("Received window type but value is 0");
                None
            } else {
                Some(ids[0])
            }
        } else {
            debug!(
                target: "get_wm_transient_for",
                "window {} did not set WM_TRANSIENT_FOR", window
            );
            None
        }
    }

    /// Checks whether the `URGENCY` flag in `WM_HINTS` is set.
    fn get_urgency(&self, window: XWindowID) -> bool {
        if let Some(hints) = self.get_wm_hints(window) {
            hints.urgent()
        } else {
            false
        }
    }

    // EWMH-related operations
    /// Gets EWMH's `_NET_WM_WINDOW_TYPE`.
    fn get_window_type(&self, window: XWindowID) -> Result<Vec<String>> {
        let atom = Atom::NetWmWindowType.as_ref();

        match self.get_property(atom, window)? {
            Some(Property::Atom(atoms)) => Ok(atoms),
            None => Ok(vec![]),
            _ => Err(XError::InvalidPropertyData(
                "Expected Atom type for get_window_type".into(),
            )),
        }
    }

    /// Gets EWMH's `_NET_WM_STATE`.
    fn get_window_states(&self, window: XWindowID) -> Result<Vec<String>> {
        let atom = Atom::NetWmState.as_ref();

        match self.get_property(atom, window)? {
            Some(Property::Atom(atoms)) => Ok(atoms),
            None => Ok(vec![]),
            _ => Err(XError::InvalidPropertyData(
                "Expected Atom type for get_window_states".into(),
            )),
        }
    }

    /// Sets the _NET_SUPPORTED property on the root window.
    ///
    /// This indicates the protocols supported by the window manager.
    fn set_supported(&self, atoms: &[Atom]) -> Result<()> {
        self.set_property(
            self.get_root().id,
            Atom::NetSupported.as_ref(),
            Property::Atom(atoms.iter().map(|a| a.to_string()).collect()),
        )
    }

    /// Sets `_NET_WM_STATE` to the given atoms on the selected window.
    fn set_wm_state(&self, window: XWindowID, atoms: &[XAtom]) {
        let atoms = atoms
            .iter()
            .map(|s| self.lookup_atom(*s).unwrap_or_else(|_| String::new()))
            .filter(|s| !s.is_empty())
            .collect();
        self.set_property(window, Atom::NetWmState.as_ref(), Property::Atom(atoms))
            .unwrap_or_else(|_| error!("failed to set wm state"));
    }

    /// Returns whether a WindowManager should manage a window.
    fn should_manage(&self, window: XWindowID) -> bool {
        let win_type = match self.get_window_type(window) {
            Ok(atoms) => atoms
                .into_iter()
                .map(|s| Atom::from_str(&s))
                .filter_map(|a| a.ok())
                .collect::<Vec<Atom>>(),
            Err(_) => return true,
        };

        !UNMANAGED_WINDOW_TYPES.iter().any(|a| win_type.contains(a))
    }

    /// Returns whether a WindowManager should set a window to floating.
    ///
    /// Can accept user-specified classes that should float.
    fn should_float(&self, window: XWindowID, float_classes: &[String]) -> bool {
        let (_, class) = self.get_wm_class(window);

        if float_classes.iter().any(|s| *s == class) {
            return true;
        }

        let win_type = match self.get_window_type(window) {
            Ok(atoms) => atoms
                .into_iter()
                .map(|s| Atom::from_str(&s))
                .filter_map(|s| s.ok())
                .collect::<Vec<Atom>>(),
            Err(_) => return true,
        };

        AUTO_FLOAT_WINDOW_TYPES.iter().any(|a| win_type.contains(a))
    }
}

/// A type that can send RandR requests.
/// 
/// For more information on the overall trait structure,
/// see the module-level documentation.
/// 
/// Its only requirement now is that it can initialize RandR,
/// but this is subject to change in the future.
pub trait RandR {
    /// Initialize RandR.
    /// 
    /// It should return a `u8` corresponding to RandR's event base.
    fn initialize_randr(&self) -> Result<u8>;
}

/// A type that can send XKB requests.
/// 
/// For more information on the overall trait structure,
/// see the module-level documentation.
/// 
/// Its only requirement now is that it can initialize RandR,
/// but this is subject to change in the future.
pub trait Xkb {
    /// Initialize XKB.
    /// 
    /// XKB does not make use of any specific event bases, so
    /// this method should just indicate whether initialization
    /// succeeded.
    fn initialize_xkb(&self) -> Result<()>;
}

/// Abstracts over methods that all XConn implementations use internally.
#[allow(dead_code)]
pub(crate) trait XConnInner: XConn {}