//! Implementation of the XConn trait for X11RBConn.
//!
use x11rb::connection::Connection;
use x11rb::protocol::{
    randr::ConnectionExt as RConnectionExt,
    xproto::{self, ConnectionExt as XConnectionExt, EventMask, GrabMode},
};

use byteorder::{LittleEndian, WriteBytesExt};

use tracing::instrument;
use tracing::{error, warn};

use super::Initialized;
use crate::bindings::{Keybind, Mousebind};
use crate::types::{Rectangle, Logical};
use crate::platform::x::{
    types::{ClientAttrs, ClientConfig},
    core::{
        PointerQueryReply, Result, WindowClass, XAtom, XConn,
        XCore, XError, XWindow, XWindowID, Xid, XOutput
    },
    event::{ClientMessageData, ClientMessageEvent, XEvent},
    input::MODIFIERS,
    property::*,
    Atom,
};

use super::X11RBConn;

macro_rules! root_button_grab_mask {
    () => {
        EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE
    };
}

macro_rules! root_pointer_grab_mask {
    () => {
        EventMask::BUTTON_RELEASE | EventMask::BUTTON_MOTION
    };
}

impl XConn for X11RBConn<Initialized> {}

impl XCore for X11RBConn<Initialized> {
    // General X server operations
    #[cfg_attr(
        debug_assertions,
        instrument(target = "xconn", level = "trace", skip(self))
    )]
    fn poll_next_event(&self) -> Result<Option<XEvent>> {
        self.conn.flush()?;

        let event = self.conn.wait_for_event()?;
        Ok(Some(self.process_raw_event(event)?))
    }

    fn get_root(&self) -> XWindow {
        self.root
    }

    fn get_geometry(&self, window: XWindowID) -> Result<Rectangle<Logical>> {
        self.get_geometry_inner(window)
    }

    fn query_tree(&self, window: XWindowID) -> Result<Vec<XWindowID>> {
        trace!("Querying tree");

        // peak funcprog
        Ok(self
            .conn
            .query_tree(*window)?
            .reply()?
            .children
            .into_iter()
            .map(Xid)
            .collect())
    }

    fn query_pointer(&self, window: XWindowID) -> Result<PointerQueryReply> {
        let reply = self.conn.query_pointer(*window)?.reply()?;

        Ok(PointerQueryReply {
            same_screen: reply.same_screen,
            root: Xid(reply.root),
            child: Xid(reply.child),
            root_x: reply.root_x as i32,
            root_y: reply.root_x as i32,
            win_x: reply.win_x as i32,
            win_y: reply.win_y as i32,
            mask: reply.mask.into(),
        })
    }

    #[cfg_attr(
        debug_assertions,
        instrument(target = "xconn", level = "trace", skip(self))
    )]
    fn all_outputs(&self) -> Result<Vec<XOutput>> {
        let check_id = self.check_win()?;
        self.conn.flush()?;

        let res = self.conn.randr_get_screen_resources(*check_id)?.reply()?;
        let info = self.conn.randr_get_screen_info(*check_id)?.reply()?;

        // extra-peak funcprog B)
        // i'll never make something this beautiful again.
        let crtcs: Vec<_> = res
            .crtcs
            .iter()
            // for each crtc, get its info
            .flat_map(|c| self.conn.randr_get_crtc_info(*c, 0))
            // unwrap the ok value and get the reply
            .flat_map(|r| r.reply())
            // assign it an index
            .enumerate()
            // construct screen
            .map(|(i, r)| {
                let geom = Rectangle::new(r.x as i32, r.y as i32, r.height as i32, r.width as i32);
                XOutput::new(i as i32, Xid(info.root), geom)
            })
            .filter(|s| s.true_geom.size.width > 0)
            .collect();

        self.conn.destroy_window(*check_id)?.check()?;

        if crtcs.is_empty() {
            Err(XError::NoScreens)
        } else {
            Ok(crtcs)
        }
    }

    fn atom(&self, atom: &str) -> Result<XAtom> {
        if let Some(known) = self.atoms().retrieve(atom) {
            return Ok(known);
        }
        trace!("Interning atom {}", atom);
        let x = self.conn.intern_atom(false, atom.as_bytes())?.reply()?;
        trace!("Atom name: {}, atom: {}", atom, x.atom);
        Ok(Xid(x.atom))
    }

    fn lookup_atom(&self, atom: XAtom) -> Result<String> {
        trace!("Looking up atom {}", atom);
        if let Some(name) = self.atoms().retrieve_by_value(atom) {
            trace!("Got name {}", name);
            return Ok(name);
        }
        trace!("Name not known, looking up via X connection");
        let name = String::from_utf8(self.conn.get_atom_name(*atom)?.reply()?.name)?;

        trace!("Got name {}", name);
        if let Ok(mut atoms) = self.atoms.try_borrow_mut() {
            atoms.insert(&name, atom);
        } else {
            warn!("Attempted borrow failed, could not intern atom {}", name);
        }

        Ok(name)
    }

    fn lookup_interned_atom(&self, name: &str) -> Option<XAtom> {
        trace!("Looking up interned atom name {}", name);
        self.atoms().retrieve(name)
    }

    fn grab_keyboard(&self) -> Result<()> {
        trace!("Grabbing keyboard");
        let _ = self
            .conn
            .grab_keyboard(
                false,
                x11rb::NONE,
                x11rb::CURRENT_TIME,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )
            .map_err(|_| XError::ServerError("Unable to grab keyboard".into()))?
            .reply()?
            .status;
        //todo: handle the status
        Ok(())
    }

    fn ungrab_keyboard(&self) -> Result<()> {
        trace!("Ungrabbing kayboard");

        self.conn
            .ungrab_keyboard(x11rb::CURRENT_TIME)
            .map_err(|_| XError::ServerError("Unable to ungrab keyboard".into()))?
            .check()?;

        Ok(())
    }

    fn grab_key(&self, kb: Keybind, window: XWindowID) -> Result<()> {
        trace!("Grabbing key {} for window {}", kb.code, window);

        for m in MODIFIERS.iter() {
            self.conn
                .grab_key(
                    false,
                    *window,
                    (kb.modmask | *m).into(),
                    kb.code,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )
                .map_err(|_| {
                    XError::ServerError(format!(
                        "Unable to grab key {} for window {}",
                        kb.code, window
                    ))
                })?
                .check()?;
        }

        Ok(())
    }

    fn ungrab_key(&self, kb: Keybind, window: XWindowID) -> Result<()> {
        //let code = KeySymbols::new(&self.conn).get_keycode(kb.keysym).next();

        self.conn
            .ungrab_key(kb.code, *window, kb.modmask.into())
            .map_err(|_| {
                XError::ServerError(format!(
                    "Unable to ungrab key {} for window {}",
                    kb.code, window
                ))
            })?
            .check()?;
        Ok(())
    }

    fn grab_button(&self, mb: Mousebind, window: XWindowID, confine: bool) -> Result<()> {
        trace!("Grab button {:?} for window: {}", mb.button, window);

        for m in MODIFIERS.iter() {
            self.conn
                .grab_button(
                    false,
                    *window,
                    root_button_grab_mask!(),
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                    if confine { *window } else { x11rb::NONE },
                    x11rb::NONE,
                    mb.button.into(),
                    (mb.modmask | *m).into(),
                )
                .map_err(|_| {
                    XError::ServerError(format!(
                        "Unable to grab button {:?} for window {}",
                        mb.button, window
                    ))
                })?
                .check()?;
        }

        Ok(())
    }

    fn ungrab_button(&self, mb: Mousebind, window: XWindowID) -> Result<()> {
        trace!("Ungrabbing button {:?} for window {}", mb.button, window);

        Ok(self
            .conn
            .ungrab_button(mb.button.into(), *window, mb.modmask.into())
            .map_err(|_| {
                XError::ServerError(format!(
                    "Unable to ungrab button {:?} for window {}",
                    mb.button, window
                ))
            })?
            .check()?)
    }

    fn grab_pointer(&self, winid: XWindowID, _mask: u32) -> Result<()> {
        trace!("Grabbing pointer for window: {:?}", winid);

        let _ = self
            .conn
            .grab_pointer(
                false,
                *winid,
                root_pointer_grab_mask!(),
                GrabMode::ASYNC,
                GrabMode::ASYNC,
                x11rb::NONE,
                x11rb::NONE,
                x11rb::CURRENT_TIME,
            )?
            .reply()?;

        Ok(())
    }

    fn ungrab_pointer(&self) -> Result<()> {
        trace!("Ungrabbing pointer");

        self.conn.ungrab_pointer(x11rb::CURRENT_TIME)?.check()?;

        Ok(())
    }

    #[instrument(target = "xconn", level = "trace", skip(self))]
    fn create_window(&self, ty: WindowClass, geom: Rectangle<Logical>, managed: bool) -> Result<XWindowID> {
        use xproto::{CreateWindowAux, WindowClass as XWindowClass};

        let (ty, bwidth, class, mut data, depth, visualid) = match ty {
            WindowClass::CheckWin => (
                None,
                0,
                XWindowClass::INPUT_OUTPUT,
                CreateWindowAux::new(),
                0,
                0,
            ),
            WindowClass::InputOnly => (
                None,
                0,
                XWindowClass::INPUT_ONLY,
                CreateWindowAux::new(),
                0,
                0,
            ),
            WindowClass::InputOutput(a, b) => {
                let mid = self.conn.generate_id()?;
                let screen = self.screen(self.idx)?;
                let depth = self.depth(screen)?;
                let visual = self.visual_type(depth)?;

                self.conn.create_colormap(
                    xproto::ColormapAlloc::NONE,
                    mid,
                    screen.root,
                    visual.visual_id,
                )?;

                (
                    Some(a),
                    b,
                    XWindowClass::INPUT_OUTPUT,
                    CreateWindowAux::new()
                        .border_pixel(0x00000000) //fixme: see above
                        .colormap(mid)
                        .event_mask(EventMask::EXPOSURE | EventMask::KEY_PRESS),
                    depth.depth,
                    visual.visual_id,
                )
            }
        };

        if !managed {
            data = data.override_redirect(1 /* true */);
        }
        let wid = Xid(self.conn.generate_id()?);
        self.conn.create_window(
            depth,
            *wid,
            *self.root.id,
            geom.point.x as i16,
            geom.point.y as i16,
            geom.size.width as u16,
            geom.size.height as u16,
            bwidth as u16,
            class,
            visualid,
            &data,
        )?;

        if let Some(a) = ty {
            let net_name = Atom::NetWmWindowType.as_ref();
            self.set_property(wid, net_name, Property::Atom(vec![a.as_ref().into()]))?;
        }

        self.conn.flush()?;

        Ok(wid)
    }

    // Window-related operations
    fn map_window(&self, window: XWindowID) -> Result<()> {
        trace!("Mapping window {}", window);

        let cookie = self.conn.map_window(*window)?.check();
        if let Err(e) = cookie {
            error!("Could not map window {}: {}", window, e);
            Err(e.into())
        } else {
            Ok(())
        }
    }

    fn unmap_window(&self, window: XWindowID) -> Result<()> {
        trace!("Unmapping window {}", window);

        let cookie = self.conn.unmap_window(*window)?.check();
        if let Err(e) = cookie {
            error!("Could not unmap window {}: {}", window, e);
            Err(e.into())
        } else {
            Ok(())
        }
    }

    fn destroy_window(&self, window: XWindowID) -> Result<()> {
        let atom = Atom::WmDeleteWindow.as_ref();
        if self.win_supports(atom, window) {
            trace!("Destroying via ICCCM WM_DELETE_WINDOW");
            let atomval = self.atom(atom)?;
            let event = ClientMessageEvent {
                window,
                data: ClientMessageData::U32([*atomval, 0, 0, 0, 0]),
                type_: atomval,
            };
            self.send_client_message(window, event)
        } else {
            trace!("Destroying via the destroy window request");
            self.conn.destroy_window(*window)?.check()?;
            Ok(())
        }
    }

    fn send_client_message(&self, window: XWindowID, data: ClientMessageEvent) -> Result<()> {
        use xproto::ClientMessageData as XClientMessageData;
        use ClientMessageData::*;

        trace!("Sending client message to window {}", window);

        let (format, to_send) = match data.data {
            U8(bytes) => (8, XClientMessageData::from(bytes)),
            U16(words) => (16, XClientMessageData::from(words)),
            U32(dwords) => (32, XClientMessageData::from(dwords)),
        };

        let event = xproto::ClientMessageEvent::new(format, *window, *data.type_, to_send);

        Ok(self
            .conn
            .send_event(false, *window, EventMask::NO_EVENT, event)?
            .check()?)
    }

    fn set_input_focus(&self, window: XWindowID) -> Result<()> {
        trace!("Setting focus for window {}", window);
        self.conn
            .set_input_focus(
                xproto::InputFocus::POINTER_ROOT,
                *window,
                x11rb::CURRENT_TIME,
            )?
            .check()?; //* FIXME: use the error
        Ok(())
    }

    fn set_geometry(&self, window: XWindowID, geom: Rectangle<Logical>) -> Result<()> {
        self.configure_window(
            window,
            &[ClientConfig::Resize {
                h: geom.size.height,
                w: geom.size.width,
            }],
        )?;

        self.configure_window(
            window,
            &[ClientConfig::Move {
                x: geom.point.x,
                y: geom.point.y,
            }],
        )?;

        Ok(())
    }

    fn set_property(&self, window: XWindowID, prop: &str, data: Property) -> Result<()> {
        use Property::*;

        // replace the property
        let mode = xproto::PropMode::REPLACE;
        // get the name of the property
        let prop = self.atom(prop)?;

        /* (type of property, format (bytes), actual data) */
        let (ty, format, data) = match data {
            Atom(atoms) => (
                xproto::AtomEnum::ATOM,
                32,
                atoms
                    .iter()
                    .map(|a| self.atom(a).unwrap_or(Xid(0)))
                    .collect(),
            ),
            Cardinal(card) => (xproto::AtomEnum::CARDINAL, 32, vec![Xid(card)]),
            String(strs) | UTF8String(strs) => {
                return {
                    let string = strs.join("\0");
                    self.conn
                        .change_property(
                            mode,
                            *window,
                            *prop,
                            xproto::AtomEnum::STRING,
                            8, //format
                            string.as_bytes().len() as u32,
                            string.as_bytes(),
                        )?
                        .check()?;
                    Ok(())
                };
            }
            Window(ids) => (xproto::AtomEnum::WINDOW, 32, ids),
            WMHints(_) | WMSizeHints(_) => {
                return Err(XError::OtherError(
                    "Modifying WM_HINTS or WM_SIZE_HINTS is not supported".into(),
                ))
            }
            _ => {
                return Err(XError::InvalidPropertyData(
                    "cannot convert non-standard types".into(),
                ))
            }
        };

        let data_len = data.len();

        let mut new_data = Vec::<u8>::with_capacity(data_len * 4);
        for dword in data {
            new_data.write_u32::<LittleEndian>(*dword)?;
        }

        if new_data.len() % 4 != 0 {
            return Err(XError::ConversionError);
        }

        Ok(self
            .conn
            .change_property(mode, *window, *prop, ty, format, data_len as u32, &new_data)?
            .check()?)
    }

    fn get_property(&self, prop: &str, window: XWindowID) -> Result<Option<Property>> {
        let atom = self.atom(prop)?;
        self.get_prop_atom(atom, window)
    }

    fn set_root_scr(&mut self, _scr: i32) {
        todo!()
    }

    fn change_window_attributes(&self, window: XWindowID, attrs: &[ClientAttrs]) -> Result<()> {
        trace!("Changing window attributes");
        let attrs = super::convert::convert_cws(attrs);
        Ok(self
            .conn
            .change_window_attributes(*window, &attrs)?
            .check()?)
    }

    fn configure_window(&self, window: XWindowID, attrs: &[ClientConfig]) -> Result<()> {
        trace!("Configuring window {} with attrs {:?}", window, attrs);
        for attr in attrs {
            let attr2 = attr.into();
            trace!("{:?}", attr2);
            self.conn.configure_window(*window, &attr2)?.check()?;
        }
        Ok(())
    }

    fn reparent_window(&self, window: XWindowID, parent: XWindowID) -> Result<()> {
        trace!("Reparenting window {} under parent {}", window, parent);

        Ok(self
            .conn
            .reparent_window(
                *window, *parent, 0, 0, //* FIXME: placeholder values */ */
            )?
            .check()?)
    }
}
