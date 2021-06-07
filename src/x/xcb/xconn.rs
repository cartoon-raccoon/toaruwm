//use xcb_util::keysyms::KeySymbols;
use xcb::{
    ClientMessageData as XCBClientMsgData,
    ClientMessageEvent as XCBClientMsgEvent,
};

use crate::x::{
    xproto,
    core::{
        XWindowID, Result, XConn, XError,
        PointerQueryReply,
    }, 
    event::{
        XEvent, 
        ClientMessageData,
        ClientMessageEvent,
    },
    property::*,
};
use crate::core::{Screen, Client};
use crate::types::{
    Atom, Geometry,
    ClientAttrs,
    ClientConfig,
};
use crate::keybinds::{Keybind, Mousebind};
use super::XCBConn;

const MAX_LONG_LENGTH: u32 = 1024;

impl XConn for XCBConn {
    // General X server operations
    fn poll_next_event(&self) -> Result<Option<XEvent>> {
        self.conn.flush();

        if let Some(event) = self.conn.poll_for_event() {
            Ok(Some(self.process_raw_event(event)?))
        } else {
            Ok(self.conn.has_error().map(|_| None)?)
        }
    }

    fn get_root(&self) -> XWindowID {
        self.root
    }

    fn get_geometry(&self, window: XWindowID) -> Result<Geometry> {
        debug!("Getting geometry for window {}", window);
        Ok(xcb::get_geometry(&self.conn, window).get_reply()
            .map(|ok| Geometry {
                    x: ok.x() as i32, 
                    y: ok.y() as i32, 
                    height: ok.height() as u32, 
                    width: ok.width() as u32,
                }
            )?
        )
    }

    fn query_tree(&self, window: XWindowID) -> Result<Vec<XWindowID>> {
        debug!("Querying tree");

        Ok(xcb::query_tree(&self.conn, window).get_reply()
        .map(|ok| ok.children().to_owned())?)
    }

    fn query_pointer(&self, window: XWindowID) -> Result<PointerQueryReply> {
        let reply = xcb::query_pointer(&self.conn, window).get_reply()?;

        Ok(PointerQueryReply {
            same_screen: reply.same_screen(),
            root: reply.root(),
            child: reply.child(),
            root_x: reply.root_x() as i32,
            root_y: reply.root_x() as i32,
            win_x: reply.win_x() as i32,
            win_y: reply.win_y() as i32,
            mask: reply.mask(),
        })
    }

    fn all_outputs(&self) -> Result<Vec<Screen>> {
        //todo: randr shit
        todo!()
    }

    fn atom(&self, atom: &str) -> Result<Atom> {
        if let Some(known) = self.atoms.retrieve(atom) {
            return Ok(known)
        }
        debug!("Interning atom {}", atom);
        let x = xcb::intern_atom(&self.conn, false, atom).get_reply()?;
        debug!("Atom name: {}, atom: {}", atom, x.atom());
        Ok(x.atom())
    }

    fn lookup_atom(&self, atom: Atom) -> Result<String> {
        debug!("Looking up atom {}", atom);
        if let Some(name) = self.atoms.retrieve_by_value(atom) {
            return Ok(name)
        }
        debug!("Name not known, looking up via X connection");
        Ok(xcb::get_atom_name(&self.conn, atom).get_reply()?.name().into())
    }

    fn lookup_interned_atom(&self, name: &str) -> Option<Atom> {
        debug!("Looking up interned atom name {}", name);
        self.atoms.retrieve(&name.to_string())
    }

    fn grab_keyboard(&self) -> Result<()> {
        debug!("Grabbing keyboard");
        let _ = xcb::grab_keyboard(
            &self.conn,
            false,
            xcb::NONE,
            xcb::CURRENT_TIME,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        ).get_reply().map_err(|_|
            XError::ServerError(
                format!("Unable to grab keyboard")
            )
        );
        Ok(())
    }

    fn ungrab_keyboard(&self) -> Result<()> {
        debug!("Ungrabbing kayboard");
        let _ = xcb::ungrab_keyboard(&self.conn, xcb::CURRENT_TIME)
        .request_check()
        .map_err(|_| XError::ServerError(format!("Unable to ungrab keyboard")))?;
        Ok(())

    }

    fn grab_key(&self, kb: Keybind, window: XWindowID) -> Result<()> {
        debug!("Grabbing key {} for window {}", kb.code, window);

        //let code = KeySymbols::new(&self.conn).get_keycode(kb.keysym).next();

        
        xcb::grab_key(
            &self.conn,
            false,
            window,
            kb.modmask.into(),
            kb.code,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        ).request_check().map_err(|_|
            XError::ServerError(
                format!("Unable to grab key {} for window {}", kb.code, window)
            )
        )?;
        Ok(())
    }

    fn ungrab_key(&self, kb: Keybind, window: XWindowID) -> Result<()> {
        //let code = KeySymbols::new(&self.conn).get_keycode(kb.keysym).next();

        xcb::ungrab_key(
            &self.conn,
            kb.code,
            window,
            kb.modmask.into(),
        ).request_check().map_err(|_|
            XError::ServerError(
                format!("Unable to ungrab key {} for window {}", 
                kb.code, window)
            )
        )?;
        Ok(())

    }

    fn grab_button(&self, mb: Mousebind, window: XWindowID, confine: bool) -> Result<()> {
        debug!("Grab button {:?} for window: {}", mb.button, window);

        xcb::grab_button(
            &self.conn, 
            false, 
            window, 
            mb.modmask as u16, 
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
            if confine { window } else { xcb::NONE },
            xcb::NONE,
            mb.button.into(),
            mb.modmask.into(),
        ).request_check().map_err(|_|
            XError::ServerError(
                format!("Unable to grab button {:?} for window {}", mb.button, window)
            )
        )
    }

    fn ungrab_button(&self, mb: Mousebind, window: XWindowID) -> Result<()> {
        debug!("Ungrabbing button {:?} for window {}", mb.button, window);

        xcb::ungrab_button(
            &self.conn,
            mb.button.into(),
            window,
            mb.modmask.into(),
        ).request_check().map_err(|_|
            XError::ServerError(
                format!("Unable to ungrab button {:?} for window {}",
                    mb.button, window
                )
            )
        )
    }

    fn grab_pointer(&self, winid: XWindowID, mask: xproto::EventMask) -> Result<()> {
        debug!("Grabbing pointer for window: {:?}", winid);

        xcb::grab_pointer(
            &self.conn,
            false,
            winid,
            mask as u16,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::NONE,
            xcb::NONE,
            xcb::CURRENT_TIME,
        ).get_reply()?;

        Ok(())
    }

    fn ungrab_pointer(&self) -> Result<()> {
        debug!("Ungrabbing pointer");

        xcb::ungrab_pointer(&self.conn, xcb::CURRENT_TIME).request_check()?;

        Ok(())
    }

    // Window-related operations
    fn map_window(&self, window: XWindowID) -> Result<()> {
        debug!("Mapping window {}", window);

        let cookie = xcb::map_window(&self.conn, window);
        if let Err(e) = cookie.request_check() {
            error!("Could not map window {}: {}", window, e);
            Err(e.into())
        } else {
            Ok(())
        }
    }

    fn unmap_window(&self, window: XWindowID) -> Result<()> {
        debug!("Unmapping window {}", window);

        let cookie = xcb::unmap_window(&self.conn, window);
        if let Err(e) = cookie.request_check() {
            error!("Could not unmap window {}: {}", window, e);
            Err(e.into())
        } else {
            Ok(())
        }
    }

    fn destroy_window(&self, window: &Client) -> Result<()> {
        let atom = self.atom("WM_DELETE_WINDOW")?;
        if window.supports(atom) {
            debug!("Destroying via ICCCM WM_DELETE_WINDOW");
            let event = ClientMessageEvent {
                window: window.id(),
                data: ClientMessageData::U32([atom, 0, 0, 0, 0]),
                type_: atom,
            };
            return self.send_client_message(window.id(), event)
        } else {
            debug!("Destroying via xcb::destroy_window");
            xcb::destroy_window(&self.conn, window.id()).request_check()?;
        }
        Ok(())
    }

    fn send_client_message(&self, window: XWindowID, data: ClientMessageEvent) -> Result<()> {
        use ClientMessageData::*;

        debug!("Sending client message to window {}", window);

        let (to_send, format) = match data.data {
            U8(bytes) => (XCBClientMsgData::from_data8(bytes), 8),
            U16(words) => (XCBClientMsgData::from_data16(words), 16),
            U32(dwords) => (XCBClientMsgData::from_data32(dwords), 32),
        };

        let client_msg = XCBClientMsgEvent::new(
            format, window, data.type_, to_send
        );

        Ok(xcb::send_event(
            &self.conn,
            false,
            window,
            xcb::EVENT_MASK_NO_EVENT,
            &client_msg,
        ).request_check()?)
    }

    fn set_input_focus(&self, window: XWindowID) {
        debug!("Setting focus for window {}", window);
        xcb::set_input_focus(&self.conn, 
            xcb::INPUT_FOCUS_POINTER_ROOT as u8, 
            window, xcb::CURRENT_TIME
        );
    }

    fn set_geometry(&self, window: XWindowID, geom: Geometry) -> Result<()> {
        self.configure_window(window, &[ClientConfig::Resize {
            h: geom.height, w: geom.width
        }])?;

        self.configure_window(window, &[ClientConfig::Move {
            x: geom.x, y: geom.y
        }])?;

        Ok(())
    }

    fn set_property(&self, window: XWindowID, prop: &str, data: Property) -> Result<()> {
        use Property::*;

        let mode = xcb::PROP_MODE_REPLACE as u8;
        let atom = self.atom(prop)?;

        let (ty, data) = match data {
            Atom(atoms) => (
                xcb::ATOM_ATOM, 
                atoms
                .iter()
                .map(|a| self.atom(&a).unwrap_or(0))
                .collect()
            ),
            Cardinal(card) => (xcb::ATOM_CARDINAL, vec![card]),
            String(strs) | UTF8String(strs) => {
                return Ok(
                    xcb::change_property_checked(
                        &self.conn,
                        mode,
                        window,
                        atom,
                        xcb::ATOM_STRING,
                        8,
                        strs.join("\0").as_bytes()
                    ).request_check()?
                )
            }
            Window(ids) => (xcb::ATOM_WINDOW, ids),
            WMHints(_) | WMSizeHints(_) => {
                return Err(
                    XError::OtherError(
                        "Modifying WM_HINTS or WM_SIZE_HINTS is not supported".into()
                    )
                )
            }
            _ => return Err(
                XError::InvalidPropertyData("cannot convert non-standard types".into())
            ),
        };

        Ok(xcb::change_property_checked(
            &self.conn,
            mode,
            window,
            atom,
            ty,
            32, &data
        ).request_check()?)
    }

    fn get_prop_str(&self, prop: &str, window: XWindowID) -> Result<Property> {
        let atom = self.atom(prop)?;
        self.get_prop_atom(atom, window)
    }

    fn get_prop_atom(&self, prop: Atom, window: XWindowID) -> Result<Property> {
        let r = xcb::get_property(
            &self.conn,
            false,
            window,
            prop,
            xcb::ATOM_ANY,
            // start at offset 0
            0, 
            // allow for up to 4 * MAX_LONG_LENGTH bytes of information
            MAX_LONG_LENGTH,
        ).get_reply()?;
        let atom_name = self.lookup_atom(r.type_())?;

        Ok(match atom_name.as_str() {
            "ATOM" => Property::Atom(
                r.value()
                    .iter()
                    .map(|a| self.lookup_atom(*a).unwrap_or("".into()))
                    .collect::<Vec<String>>()
            ),
            "CARDINAL" => Property::Cardinal(r.value()[0]),
            "STRING" => Property::String(
                String::from_utf8_lossy(&r.value().to_vec())
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect()
            ),
            "UTF8_STRING" => Property::String(
                String::from_utf8(r.value().to_vec())?
                    .trim_matches('\0')
                    .split('\0')
                    .map(|a| a.to_string())
                    .collect()
            ),
            "WINDOW" => Property::Window(r.value().to_vec()),
            "WM_HINTS" => Property::WMHints(
                WmHints::try_from_bytes(r.value())?
            ),
            "WM_SIZE_HINTS" => Property::WMSizeHints(
                WmSizeHints::try_from_bytes(r.value())?
            ),
            n @ _ => {
                if n == "WM_STATE" {
                    debug!("Type is WM_STATE");
                }
                match r.format() {
                    8 => Property::U8List(
                        r.value::<u8>().into()
                    ),
                    16 => Property::U16List(
                        r.value::<u16>().into()
                    ),
                    32 => Property::U32List(
                        r.value::<u32>().into()
                    ),
                    n @ _ => {
                        return Err(
                            XError::InvalidPropertyData(
                                format!("received format {}", n)
                            )
                        )
                    },
                }
            }
        })
    }

    fn set_root_scr(&mut self, _scr: i32) {
        todo!()
    }

    fn change_window_attributes(&self, window: XWindowID, attrs: &[ClientAttrs]) -> Result<()> {
        debug!("Changing window attributes");
        for attr in attrs {
            let attr2: Vec<_> = attr.into();
            xcb::change_window_attributes_checked(&self.conn, window, &attr2).request_check()?;
        }
        Ok(())
    }

    fn configure_window(&self, window: XWindowID, attrs: &[ClientConfig]) -> Result<()> {
        debug!("Configuring window {}", window);
        for attr in attrs {
            let attr2: Vec<_> = attr.into();
            xcb::configure_window_checked(&self.conn, window, &attr2).request_check()?
        }
        Ok(())
    }

    fn reparent_window(&self, window: XWindowID, parent: XWindowID) -> Result<()> {
        debug!("Reparenting window {} under parent {}", window, parent);

        Ok(xcb::reparent_window(
            &self.conn,
            window,
            parent,
            0, 0 //placeholder values
        ).request_check()?)
    }
}