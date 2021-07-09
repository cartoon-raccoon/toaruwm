//use std::convert::TryFrom;
use std::collections::HashMap;
use std::str::FromStr;

use thiserror::Error;

use strum::*;
use strum_macros::EnumIter;

use super::core::XAtom;

// shamelessly stolen from:
// https://github.com/sminez/penrose/blob/develop/src/core/xconnection/atom.rs
//
// thanks dude, and sorry for stealing your stuff.

/// ToaruWM internal representations of X atoms.
/// 
/// This allows for some measure of type safety around dealing with atoms.
#[derive(AsRefStr, Display, EnumString, EnumIter, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Atom {
    /// ATOM
    #[strum(serialize = "ATOM")]
    Atom,
    /// ATOM_WINDOW
    #[strum(serialize = "ATOM_WINDOW")]
    Window,
    /// ATOM_CARDINAL
    #[strum(serialize = "ATOM_CARDINAL")]
    Cardinal,
    /// MANAGER
    #[strum(serialize = "MANAGER")]
    Manager,
    /// STRING
    #[strum(serialize = "STRING")]
    String,
    /// UTF8_STRING
    #[strum(serialize = "UTF8_STRING")]
    UTF8String,
    /// COMPOUND_TEXT
    #[strum(serialize = "COMPOUND_TEXT")]
    CompoundText,
    /// WM_CLASS
    #[strum(serialize = "WM_CLASS")]
    WmClass,
    /// WM_DELETE_WINDOW
    #[strum(serialize = "WM_DELETE_WINDOW")]
    WmDeleteWindow,
    /// WM_HINTS
    #[strum(serialize = "WM_HINTS")]
    WmHints,
    /// WM_NORMAL_HINTS
    #[strum(serialize = "WM_NORMAL_HINTS")]
    WmNormalHints,
    /// WM_SIZE_HINTS
    #[strum(serialize = "WM_SIZE_HINTS")]
    WmSizeHints,
    /// WM_PROTOCOLS
    #[strum(serialize = "WM_PROTOCOLS")]
    WmProtocols,
    /// WM_STATE
    #[strum(serialize = "WM_STATE")]
    WmState,
    /// WM_NAME
    #[strum(serialize = "WM_NAME")]
    WmName,
    /// WM_ICON_NAME
    #[strum(serialize = "WM_ICON_NAME")]
    WmIconName,
    /// WM_TRANSIENT_FOR
    #[strum(serialize = "WM_TRANSIENT_FOR")]
    WmTransientFor,
    /// WM_TAKE_FOCUS
    #[strum(serialize = "WM_TAKE_FOCUS")]
    WmTakeFocus,
    /// _NET_ACTIVE_WINDOW
    #[strum(serialize = "_NET_ACTIVE_WINDOW")]
    NetActiveWindow,
    /// _NET_CLIENT_LIST
    #[strum(serialize = "_NET_CLIENT_LIST")]
    NetClientList,
    /// _NET_CLIENT_LIST
    #[strum(serialize = "_NET_CLIENT_LIST_STACKING")]
    NetClientListStacking,
    /// _NET_CURRENT_DESKTOP
    #[strum(serialize = "_NET_CURRENT_DESKTOP")]
    NetCurrentDesktop,
    /// _NET_DESKTOP_NAMES
    #[strum(serialize = "_NET_DESKTOP_NAMES")]
    NetDesktopNames,
    /// _NET_NUMBER_OF_DESKTOPS
    #[strum(serialize = "_NET_NUMBER_OF_DESKTOPS")]
    NetNumberOfDesktops,
    /// _NET_SUPPORTED
    #[strum(serialize = "_NET_SUPPORTED")]
    NetSupported,
    /// _NET_SUPPORTING_WM_CHECK
    #[strum(serialize = "_NET_SUPPORTING_WM_CHECK")]
    NetSupportingWmCheck,
    /// _NET_SYSTEM_TRAY_OPCODE
    #[strum(serialize = "_NET_SYSTEM_TRAY_OPCODE")]
    NetSystemTrayOpcode,
    /// _NET_SYSTEM_TRAY_ORIENTATION
    #[strum(serialize = "_NET_SYSTEM_TRAY_ORIENTATION")]
    NetSystemTrayOrientation,
    /// _NET_SYSTEM_TRAY_ORIENTATION_HORZ
    #[strum(serialize = "_NET_SYSTEM_TRAY_ORIENTATION_HORZ")]
    NetSystemTrayOrientationHorz,
    /// _NET_SYSTEM_TRAY_S0
    #[strum(serialize = "_NET_SYSTEM_TRAY_S0")]
    NetSystemTrayS0,
    /// _NET_WM_DESKTOP
    #[strum(serialize = "_NET_WM_DESKTOP")]
    NetWmDesktop,
    /// _NET_WM_NAME
    #[strum(serialize = "_NET_WM_NAME")]
    NetWmName,
    /// _NET_WM_STATE
    #[strum(serialize = "_NET_WM_STATE")]
    NetWmState,
    /// _NET_WM_STATE_FULLSCREEN
    #[strum(serialize = "_NET_WM_STATE_FULLSCREEN")]
    NetWmStateFullscreen,
    /// _NET_WM_STATE_HIDDEN
    #[strum(serialize = "_NET_WM_STATE_HIDDEN")]
    NetWmStateHidden,
    /// _NET_WM_WINDOW_TYPE
    #[strum(serialize = "_NET_WM_WINDOW_TYPE")]
    NetWmWindowType,
    /// _XEMBED
    #[strum(serialize = "_XEMBED")]
    XEmbed,
    /// _XEMBED_INFO
    #[strum(serialize = "_XEMBED_INFO")]
    XEmbedInfo,

    // Window Types
    /// _NET_WM_WINDOW_TYPE_DESKTOP
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_DESKTOP")]
    NetWindowTypeDesktop,
    /// _NET_WM_WINDOW_TYPE_DOCK
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_DOCK")]
    NetWindowTypeDock,
    /// _NET_WM_WINDOW_TYPE_TOOLBAR
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_TOOLBAR")]
    NetWindowTypeToolbar,
    /// _NET_WM_WINDOW_TYPE_MENU
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_MENU")]
    NetWindowTypeMenu,
    /// _NET_WM_WINDOW_TYPE_UTILITY
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_UTILITY")]
    NetWindowTypeUtility,
    /// _NET_WM_WINDOW_TYPE_SPLASH
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_SPLASH")]
    NetWindowTypeSplash,
    /// _NET_WM_WINDOW_TYPE_DIALOG
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_DIALOG")]
    NetWindowTypeDialog,
    /// _NET_WM_WINDOW_TYPE_DROPDOWN_MENU
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_DROPDOWN_MENU")]
    NetWindowTypeDropdownMenu,
    /// _NET_WM_WINDOW_TYPE_POPUP_MENU
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_POPUP_MENU")]
    NetWindowTypePopupMenu,
    /// _NET_WM_WINDOW_TYPE_NOTIFICATION
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_NOTIFICATION")]
    NetWindowTypeNotification,
    /// _NET_WM_WINDOW_TYPE_COMBO
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_COMBO")]
    NetWindowTypeCombo,
    /// _NET_WM_WINDOW_TYPE_DND
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_DND")]
    NetWindowTypeDnd,
    /// _NET_WM_WINDOW_TYPE_NORMAL
    #[strum(serialize = "_NET_WM_WINDOW_TYPE_NORMAL")]
    NetWindowTypeNormal,
}

#[derive(Clone, Copy, Debug, Error)]
#[error("Could not get known atom from given atom {0}")]
pub struct TryFromAtomError(XAtom);

/// Clients with one of these window types will be auto floated
pub const AUTO_FLOAT_WINDOW_TYPES: &[Atom] = &[
    Atom::NetWindowTypeCombo,
    Atom::NetWindowTypeDesktop,
    Atom::NetWindowTypeDialog,
    Atom::NetWindowTypeDropdownMenu,
    Atom::NetWindowTypeNotification,
    Atom::NetWindowTypeMenu,
    Atom::NetWindowTypePopupMenu,
    Atom::NetWindowTypeToolbar,
    Atom::NetWindowTypeUtility,
];
    
/// Windows with a type in this array will not be managed
pub const UNMANAGED_WINDOW_TYPES: &[Atom] = &[
    Atom::NetWindowTypeSplash,
    Atom::NetWindowTypeDock,
    Atom::NetWindowTypeNotification,
    Atom::NetWindowTypeToolbar,
    Atom::NetWindowTypeUtility,
];

/// Currently supported EWMH atoms
pub const EWMH_SUPPORTED_ATOMS: &[Atom] = &[
    Atom::NetActiveWindow,
    Atom::NetClientList,
    Atom::NetClientListStacking,
    Atom::NetCurrentDesktop,
    Atom::NetDesktopNames,
    Atom::NetNumberOfDesktops,
    Atom::NetSupported,
    Atom::NetSupportingWmCheck,
    // Atom::NetSystemTrayS0,
    // Atom::NetSystemTrayOpcode,
    // Atom::NetSystemTrayOrientationHorz,
    Atom::NetWmName,
    Atom::NetWmState,
    Atom::NetWmStateFullscreen,
    Atom::NetWmWindowType,
];

/// A type that associates either an Atom or a String with
/// an X-defined atom.
/// 
/// This allows the user to manage known atoms conveniently.
#[derive(Default, Clone)]
pub struct Atoms {
    /// Known atoms that can be managed as their enum variants.
    known: HashMap<Atom, XAtom>,
    /// Unknown atoms that have to be managed as strings.
    interned: HashMap<String, XAtom>,
}

impl Atoms {
    pub fn new() -> Self {
        Self {
            known: HashMap::new(),
            interned: HashMap::new(),
        }
    }

    pub fn insert(&mut self, atom: &str, val: XAtom) {
        if let Ok(known) = Atom::from_str(atom) {
            self.known.insert(known, val);
        } else {
            self.interned.insert(atom.into(), val);
        }
    }

    pub fn retrieve(&self, atom: &str) -> Option<XAtom> {
        if let Ok(known) = Atom::from_str(atom) {
            self.known.get(&known).copied()
        } else {
            self.interned.get(&atom.to_string()).copied()
        }
    }

    pub fn retrieve_by_value(&self, atom: XAtom) -> Option<String> {
        if let Some((known, _)) = self.known.iter().find(|(_, v)| **v == atom) {
            Some(known.to_string())
        } else {
            self.interned.iter()
            .find(|(_, v)| **v == atom)
            .map(|(k, _)| k.clone())
        }
    }
}