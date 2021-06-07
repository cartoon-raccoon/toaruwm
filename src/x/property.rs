use crate::x::core::{
    XConn, XWindowID,
};
use crate::types::Atom;

/// X server properties.
#[derive(Debug, Clone)]
pub enum Property {
    /// a list of Atoms (u32), expressed as strings.
    Atom(Vec<String>),

    /// a cardinal number.
    Cardinal(u32),

    /// a list of strings.
    String(Vec<String>),

    /// a list of UTF-8 encoded strings.
    UTF8String(Vec<String>),

    /// a list of windows IDs.
    Window(Vec<XWindowID>),

    /// WM_HINTS.
    WMHints(WmHints),

    /// WM_SIZE_HINTS.
    WMSizeHints(SizeHints),

    /// Raw data as a vec of bytes.
    /// Returned if the format of the response is 8.
    /// 
    /// Used if the property type is not recognized by toaruwm.
    U8List(Vec<u8>),

    /// Raw data as a vec of words.
    /// Returned if the format of the response is 16.
    /// 
    /// Used of the property type is not recognized by toaruwm.
    U16List(Vec<u16>),

    /// Raw data as a vec of doublewords.
    /// Returned if the format of the response is 32.
    /// 
    /// Used of the property type is not recognized by toaruwm.
    U32List(Vec<u32>),
}

impl Property {
    /// If the property is `Self::Atoms(_), return its internal
    /// representation as a Vec of Atoms instead of Strings.
    /// 
    /// If the property is not `Self::Atoms`, None is returned.
    pub fn as_atoms<X: XConn>(&self, conn: &X) -> Option<Vec<Atom>> {
        if let Self::Atom(strings) = self {
            Some({
                let mut atoms = Vec::new();
                for s in strings {
                    atoms.push(conn.atom(s).ok()?)
                }
                atoms
            })
        } else {
            None
        }
    }
}

// generate Property::is_<var> methods
macro_rules! derive_is {
    ($name:ident, $var:pat) => {
        impl Property {
            pub fn $name(&self) -> bool {
                matches!(self, $var)
            }
        }
    }
}

derive_is!(is_atom, Self::Atom(_));
derive_is!(is_card, Self::Cardinal(_));
derive_is!(is_string, Self::String(_));
derive_is!(is_utf8str, Self::UTF8String(_));
derive_is!(is_window, Self::Window(_));
derive_is!(is_wmhints, Self::WMHints(_));
derive_is!(is_sizehints, Self::WMSizeHints(_));
derive_is!(is_u8list, Self::U8List(_));
derive_is!(is_u16list, Self::U16List(_));
derive_is!(is_u32list, Self::U32List(_));

/// The ICCCM-defined window states.
#[derive(Clone, Copy, Debug)]
pub enum WindowState {
    Normal = 1,
    Iconic = 3,
    Withdrawn = 0,
}


/// ICCCM-defined window hints (WM_HINTS).
#[derive(Debug, Clone, Copy)]
pub struct WmHints {
    pub state: WindowState,
    pub urgent: bool,
    //todo: add pixmaps
}

/// ICCCM-defined window size hints (WM_SIZE_HINTS).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SizeHints {
    pub position: Option<(i32, i32)>,
    pub size: Option<(i32, i32)>,
    pub min_size: Option<(i32, i32)>,
    pub max_size: Option<(i32, i32)>,
    pub resize: Option<(i32, i32)>,
    pub min_aspect: Option<(i32, i32)>,
    pub max_aspect: Option<(i32, i32)>,
    pub base: Option<(i32, i32)>,
    pub gravity: Option<u32>
}