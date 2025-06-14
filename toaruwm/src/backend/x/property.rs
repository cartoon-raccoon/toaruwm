//! Types for working with window properties.

use std::convert::TryFrom;
use std::fmt;

use strum::{EnumIs};

use crate::types::Point;
use crate::backend::x::core::{Result, XAtom, XConn, XError, XWindowID, Xid};

/// X server properties.
#[derive(Debug, Clone, EnumIs)]
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
    WMSizeHints(WmSizeHints),

    /// Raw data as a vec of bytes.
    /// Returned if the format of the response is 8.
    /// The property type is also provided as a String.
    ///
    /// Used if the property type is not recognized by toaruwm.
    U8List(String, Vec<u8>),

    /// Raw data as a vec of words.
    /// Returned if the format of the response is 16.
    /// The property type is also provided as a String.
    ///
    /// Used if the property type is not recognized by toaruwm.
    U16List(String, Vec<u16>),

    /// Raw data as a vec of doublewords.
    /// Returned if the format of the response is 32.
    /// The property type is also provided as a String.
    ///
    /// Used if the property type is not recognized by toaruwm.
    U32List(String, Vec<u32>),
}

impl Property {
    /// If the property is `Self::Atoms(_)`, return its internal
    /// representation as a Vec of Atoms instead of Strings.
    /// `conn` is required to contact the X server to get each atom's
    /// corresponding integer value, or to intern the atom on the
    /// server if the atom doesn't currently exist.
    ///
    /// Any errors encountered when fetching values are silently discarded.
    ///
    /// If the property is not `Self::Atoms`, None is returned.
    pub fn as_atoms<X: XConn>(&self, conn: &X) -> Option<Vec<XAtom>> {
        if let Self::Atom(strings) = self {
            Some(strings.iter().flat_map(|s| conn.atom(s)).collect())
        } else {
            None
        }
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::string::String as StdString;
        use Property::*;

        match self {
            Atom(strs) => {
                write!(f, "Atom: {:?}", strs)
            }
            Cardinal(n) => {
                write!(f, "Cardinal: {}", n)
            }
            String(strs) => {
                write!(f, "Strings: {:?}", strs)
            }
            UTF8String(strs) => {
                write!(f, "Strings: {:?}", strs)
            }
            Window(ids) => {
                write!(f, "Windows: {:?}", ids)
            }
            WMHints(hints) => {
                write!(f, "WmHints: {:#?}", hints)
            }
            WMSizeHints(hints) => {
                write!(f, "WmSizeHints: {:#?}", hints)
            }
            U8List(s, u8s) => {
                let mut out = format!("{}: [", s);

                out.push_str(
                    &u8s.iter()
                        .map(|s| format!("{:#04x}", s))
                        .collect::<Vec<StdString>>()
                        .join(","),
                );

                out.push(']');

                write!(f, "u8[] - {}", out)
            }
            U16List(s, u16s) => {
                let mut out = format!("{}: [", s);

                out.push_str(
                    &u16s
                        .iter()
                        .map(|s| format!("{:#06x}", s))
                        .collect::<Vec<StdString>>()
                        .join(","),
                );

                out.push(']');

                write!(f, "u16[] - {}", out)
            }
            U32List(s, u32s) => {
                let mut out = format!("{}: [", s);

                out.push_str(
                    &u32s
                        .iter()
                        .map(|s| format!("{:#08x}", s))
                        .collect::<Vec<StdString>>()
                        .join(","),
                );

                out.push(']');

                write!(f, "u32[] - {}", out)
            }
        }
    }
}

/// The ICCCM-defined window states.
#[derive(Clone, Copy, Debug)]
pub enum WindowState {
    /// The window is shown as normal.
    Normal = 1,
    /// The window has been "iconified".
    Iconic = 3,
    /// The window is withdrawn (unmapped).
    Withdrawn = 0,
}

impl Default for WindowState {
    fn default() -> Self {
        Self::Withdrawn
    }
}

bitflags! {

/// The flags used inside WmHints.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct WmHintsFlags: u32 {
    /// The input hint is set
    const INPUT_HINT            = 0b0000000001;
    /// The state hint is set
    const STATE_HINT            = 0b0000000010;
    /// The icon pixmap hint is set
    const ICON_PIXMAP_HINT      = 0b0000000100;
    /// The icon window hint is set
    const ICON_WINDOW_HINT      = 0b0000001000;
    /// The icon position hint is set
    const ICON_POSITION_HINT    = 0b0000010000;
    /// The icon mask hint is set
    const ICON_MASK_HINT        = 0b0000100000;
    /// The window group hint is set
    const WINDOW_GROUP_HINT     = 0b0001000000;
    //const UNUSED                = 0b0010000000;
    /// The urgency hint is set
    const URGENCY_HINT          = 0b0100000000;
}

/// The flags used inside WmSizeHints.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct WmSizeHintsFlags: u32 {
    /// User-specified x and y
    const US_POSITION   = 0b0000000001;
    /// User-specified window size
    const US_SIZE       = 0b0000000010;
    /// Program-specified position
    const P_POSITION    = 0b0000000100;
    /// Program-specified size
    const P_SIZE        = 0b0000001000;
    /// Program-specified minimum size
    const P_MIN_SIZE    = 0b0000010000;
    /// Program specified maximum size
    const P_MAX_SIZE    = 0b0000100000;
    /// Program specified resize increments
    const P_RESIZE_INC  = 0b0001000000;
    /// Program specified aspect ratios
    const P_ASPECT      = 0b0010000000;
    /// Program specified base size
    const P_BASE_SIZE   = 0b0100000000;
    /// Program specified window gravity
    const P_WIN_GRAVITY = 0b1000000000;
}

}

/// The length of the data for WM_HINTS.
const WM_HINTS_LEN: usize = 9;

/// The length of the data for WM_SIZE_HINTS.
const WM_SIZE_HINTS_LEN: usize = 18;

/// ICCCM-defined window hints (WM_HINTS).
///
/// This struct contains all fields of the WM_HINTS
/// type, but ToaruWM does not honour the following currently:
///
/// - ICON_PIXMAP
/// - ICON_WINDOW
/// - ICON_POS
/// - ICON_MASK
///
/// These fields may be supported in the future.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WmHints {
    pub(crate) flags: WmHintsFlags,
    pub(crate) accepts_input: bool,
    pub(crate) initial_state: WindowState,
    pub(crate) icon_pixmap: u32,
    pub(crate) icon_window: XWindowID,
    pub(crate) icon_pos: Point,
    pub(crate) icon_mask: u32,
    pub(crate) window_group: XWindowID,
}

impl WmHints {
    /// Returns an empty `WMHints`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Attempts to parse WmHints from a u32 slice
    /// According to the following C struct definition:
    ///
    /// ```c
    /// typedef struct {
    ///     int32_t flags;
    ///     uint32_t input;
    ///     int32_t initial_state;
    ///     xcb_pixmap_t icon_pixmap;  /* uint32_t */
    ///     xcb_window_t icon_window;  /* uint32_t */
    ///     int32_t icon_x, icon_y;
    ///     xcb_pixmap_t icon_mask;    /* uint32_t */
    ///     xcb_window_t window_group; /* uint32_t */
    /// } xcb_icccm_wm_hints_t;
    /// ```
    ///
    /// Declaration taken from
    /// [here](https://cgit.freedesktop.org/xcb/util-wm/tree/icccm/xcb_icccm.h).
    ///
    /// Returns XError::InvalidPropertyData on failure.
    pub fn try_from_bytes(raw: &[u32]) -> Result<Self> {
        TryFrom::try_from(raw)
    }

    /// Test whether `flag` is set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::x::property::{
    ///     WmHints,
    ///     WmHintsFlags,
    /// };
    ///
    /// let wm_hints = WmHints::new();
    ///
    /// /* URGENCY flag is not set */
    /// assert!(!wm_hints.is_set(WmHintsFlags::URGENCY_HINT));
    /// ```
    pub fn is_set(&self, flag: WmHintsFlags) -> bool {
        self.flags.contains(flag)
    }

    /// Test if the urgency flag is set.
    pub fn urgent(&self) -> bool {
        self.flags.contains(WmHintsFlags::URGENCY_HINT)
    }
}

impl TryFrom<&[u32]> for WmHints {
    type Error = XError;

    fn try_from(from: &[u32]) -> Result<Self> {
        use XError::*;

        if from.len() != WM_HINTS_LEN {
            return Err(InvalidPropertyData(format!(
                "expected [u32; 9], got {}",
                from.len()
            )));
        }

        let flags = WmHintsFlags::from_bits(from[0])
            .ok_or_else(|| InvalidPropertyData("invalid flags set for WmHintsFlags".into()))?;

        let accepts_input = !flags.contains(WmHintsFlags::INPUT_HINT) || from[1] > 0;

        let initial_state = match (flags.contains(WmHintsFlags::STATE_HINT), from[2]) {
            (true, 0) => WindowState::Withdrawn,
            (true, 1) => WindowState::Normal,
            (true, 3) | (true, 2) => WindowState::Iconic,
            (true, n) => {
                return Err(InvalidPropertyData(format!(
                    "expected 0, 1, or 3 for window state, got {}",
                    n
                )))
            }
            (false, _) => WindowState::Normal,
        };

        let icon_pos = Point {
            x: from[5] as i32,
            y: from[6] as i32,
        };

        Ok(WmHints {
            flags,
            accepts_input,
            initial_state,
            icon_pixmap: from[3],
            icon_window: Xid(from[4]),
            icon_pos,
            icon_mask: from[7],
            window_group: Xid(from[8]),
        })
    }
}

/// ICCCM-defined window size hints (WM_SIZE_HINTS).
///
/// ## Notes
///
/// Position and Size are outdated and only exist for
/// backwards compatibility.
///
/// This struct contains all the fields in the
/// WM_SIZE_HINTS type, but ToaruWM does not honour
/// the following flags:
///
/// - Aspect ratio
/// - Gravity
/// - Increments
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct WmSizeHints {
    pub(crate) flags: WmSizeHintsFlags,
    pub(crate) position: Option<(i32, i32)>,
    pub(crate) size: Option<(i32, i32)>,
    pub(crate) min_size: Option<(i32, i32)>,
    pub(crate) max_size: Option<(i32, i32)>,
    pub(crate) resize_inc: Option<(i32, i32)>,
    pub(crate) min_aspect: Option<(i32, i32)>,
    pub(crate) max_aspect: Option<(i32, i32)>,
    pub(crate) base_size: Option<(i32, i32)>,
    pub(crate) gravity: Option<u32>,
}

impl WmSizeHints {
    /// Returns an empty `WMSizeHints`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Attempts to parse WmSizeHints from a u32 slice
    /// according to the following C struct definition:
    ///
    /// ```c
    /// typedef struct {
    ///     uint32_t flags;
    ///     int32_t x, y;
    ///     int32_t width, height;
    ///     int32_t min_width, min_height;
    ///     int32_t max_width, max_height;
    ///     int32_t width_inc, height_inc;
    ///     int32_t min_aspect_num, min_aspect_den;
    ///     int32_t max_aspect_num, max_aspect_den;
    ///     int32_t base_width, base_height;
    ///     uint32_t win_gravity;
    /// } xcb_size_hints_t;
    /// ```
    ///
    /// Declaration taken from
    /// [here](https://cgit.freedesktop.org/xcb/util-wm/tree/icccm/xcb_icccm.h).
    ///
    /// Returns XError::InvalidPropertyData on failure.
    pub fn try_from_bytes(raw: &[u32]) -> Result<Self> {
        TryFrom::try_from(raw)
    }

    /// Test whether `flag` is set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use toaruwm::x::property::{
    ///     WmSizeHints,
    ///     WmSizeHintsFlags,
    /// };
    ///
    /// let size_hints = WmSizeHints::new();
    ///
    /// /* P_SIZE flag is not set */
    /// assert!(!size_hints.is_set(WmSizeHintsFlags::P_SIZE));
    /// ```
    pub fn is_set(&self, flag: WmSizeHintsFlags) -> bool {
        self.flags.contains(flag)
    }
}

impl TryFrom<&[u32]> for WmSizeHints {
    type Error = XError;

    fn try_from(from: &[u32]) -> Result<Self> {
        use WmSizeHintsFlags as WMSHFlags;
        use XError::*;

        if from.len() != WM_SIZE_HINTS_LEN {
            return Err(InvalidPropertyData(format!(
                "expected [u32; 18], got {}",
                from.len()
            )));
        }

        let flags = WMSHFlags::from_bits(from[0])
            .ok_or_else(|| InvalidPropertyData("invalid flags set for WmSizeHintsFlags".into()))?;

        let position =
            if flags.contains(WMSHFlags::US_POSITION) || flags.contains(WMSHFlags::P_POSITION) {
                Some((from[1] as i32, from[2] as i32))
            } else {
                None
            };

        let size = if flags.contains(WMSHFlags::US_SIZE) || flags.contains(WMSHFlags::P_POSITION) {
            Some((from[3] as i32, from[4] as i32))
        } else {
            None
        };

        let min_size = if flags.contains(WMSHFlags::P_MIN_SIZE) {
            Some((from[5] as i32, from[6] as i32))
        } else {
            None
        };

        let max_size = if flags.contains(WMSHFlags::P_MAX_SIZE) {
            Some((from[7] as i32, from[8] as i32))
        } else {
            None
        };

        let resize_inc = if flags.contains(WMSHFlags::P_RESIZE_INC) {
            Some((from[9] as i32, from[10] as i32))
        } else {
            None
        };

        // might as well directly use None for now
        let (min_aspect, max_aspect) = (None, None);

        let base_size = if flags.contains(WMSHFlags::P_BASE_SIZE) {
            Some((from[15] as i32, from[16] as i32))
        } else {
            None
        };

        let gravity = if flags.contains(WMSHFlags::P_WIN_GRAVITY) {
            Some(from[17])
        } else {
            None
        };

        Ok(WmSizeHints {
            flags,
            position,
            size,
            min_size,
            max_size,
            resize_inc,
            min_aspect,
            max_aspect,
            base_size,
            gravity,
        })
    }
}
