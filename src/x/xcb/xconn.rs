//use xcb_util::keysyms::KeySymbols;
use xcb::{
    ClientMessageData as XCBClientMsgData,
    ClientMessageEvent as XCBClientMsgEvent,
    randr,
};

use tracing::instrument;
use tracing::{trace, error};

use crate::x::{
    core::{
        XWindow, XWindowID,
        XConn, XError, XAtom,
        PointerQueryReply, Result,
        WindowClass, 
    }, 
    event::{
        XEvent, 
        ClientMessageData,
        ClientMessageEvent,
    },
    property::*,
    Atom,
};
use crate::core::Screen;
use crate::types::{
    BORDER_WIDTH,
    Geometry,
    ClientAttrs,
    ClientConfig,
};
use crate::keybinds::{Keybind, Mousebind};
use crate::util;

use super::XCBConn;

// Grab numlock separately and filter it out when receiving events
// Taken from https://github.com/sminez/penrose/blob/develop/src/xcb/api.rs.
const MODIFIERS: &[u16] = &[0, xcb::MOD_MASK_2 as u16];

impl XConn for XCBConn {
    // General X server operations
    #[instrument(target="xconn", level="trace", skip(self))]
    fn poll_next_event(&self) -> Result<Option<XEvent>> {
        self.conn.flush();

        if let Some(event) = self.conn.wait_for_event() {
            Ok(Some(self.process_raw_event(event)?))
        } else {
            Ok(self.conn.has_error().map(|_| None)?)
        }
    }

    fn get_root(&self) -> XWindow {
        self.root
    }

    fn get_geometry(&self, window: XWindowID) -> Result<Geometry> {
        trace!("Getting geometry for window {}", window);
        Ok(xcb::get_geometry(&self.conn, window).get_reply()
            .map(|ok| Geometry {
                    x: ok.x() as i32, 
                    y: ok.y() as i32, 
                    height: ok.height() as i32, 
                    width: ok.width() as i32,
                }
            )?
        )
    }

    fn query_tree(&self, window: XWindowID) -> Result<Vec<XWindowID>> {
        trace!("Querying tree");

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

    #[instrument(target="xconn", level="trace", skip(self))]
    fn all_outputs(&self) -> Result<Vec<Screen>> {
        let check_id = self.check_win()?;
        self.conn.flush();

        let res = randr::get_screen_resources(&self.conn, check_id)
            .get_reply()?;

        let info = randr::get_screen_info(&self.conn, check_id)
            .get_reply()?;
        
        let crtcs = res.crtcs().iter()
            // could do this with flat_map, but that just seems confusing

            // for each crtc, get its info
            .map(|c| randr::get_crtc_info(&self.conn, *c, 0).get_reply())
            // filter out errors
            .filter(|r| r.is_ok())
            // unwrap it
            .map(|ok| ok.unwrap())
            // assign it an index
            .enumerate()
            // construct screen
            .map(|(i, r)| {
                let geom = Geometry::new(
                    r.x() as i32, 
                    r.y() as i32,
                    r.height() as i32,
                    r.width() as i32,
                );
                Screen::new(i as i32, geom, info.root(), vec![])
            })
            .filter(|s| s.true_geom().width > 0).collect();

        xcb::destroy_window(&self.conn, check_id);

        Ok(crtcs)
    }

    fn atom(&self, atom: &str) -> Result<XAtom> {
        if let Some(known) = self.atoms().retrieve(atom) {
            return Ok(known)
        }
        trace!("Interning atom {}", atom);
        let x = xcb::intern_atom(&self.conn, false, atom).get_reply()?;
        trace!("Atom name: {}, atom: {}", atom, x.atom());
        Ok(x.atom())
    }

    fn lookup_atom(&self, atom: XAtom) -> Result<String> {
        trace!("Looking up atom {}", atom);
        if let Some(name) = self.atoms().retrieve_by_value(atom) {
            trace!("Got name {}", name);
            return Ok(name)
        }
        trace!("Name not known, looking up via X connection");
        let name = xcb::get_atom_name(&self.conn, atom).get_reply()?.name().to_string();

        if let Ok(mut atoms) = self.atoms.try_borrow_mut() {
            atoms.insert(&name, atom);
        }
        trace!("Got name {}", name);
        Ok(name)
    }

    fn lookup_interned_atom(&self, name: &str) -> Option<XAtom> {
        trace!("Looking up interned atom name {}", name);
        self.atoms().retrieve(&name.to_string())
    }

    fn grab_keyboard(&self) -> Result<()> {
        trace!("Grabbing keyboard");
        let _ = xcb::grab_keyboard(
            &self.conn,
            false,
            xcb::NONE,
            xcb::CURRENT_TIME,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        ).get_reply().map_err(|_|
            XError::ServerError(
                "Unable to grab keyboard".into()
            )
        );
        Ok(())
    }

    fn ungrab_keyboard(&self) -> Result<()> {
        trace!("Ungrabbing kayboard");
        let _ = xcb::ungrab_keyboard(&self.conn, xcb::CURRENT_TIME)
        .request_check()
        .map_err(|_| XError::ServerError("Unable to ungrab keyboard".into()))?;
        Ok(())

    }

    fn grab_key(&self, kb: Keybind, window: XWindowID) -> Result<()> {
        trace!("Grabbing key {} for window {}", kb.code, window);

        for m in MODIFIERS.iter() {
            xcb::grab_key(
                &self.conn,
                false,
                window,
                kb.modmask | m,
                kb.code,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            ).request_check().map_err(|_|
                XError::ServerError(
                    format!("Unable to grab key {} for window {}", kb.code, window)
                )
            )?;
        }
        
        Ok(())
    }

    fn ungrab_key(&self, kb: Keybind, window: XWindowID) -> Result<()> {
        //let code = KeySymbols::new(&self.conn).get_keycode(kb.keysym).next();

        xcb::ungrab_key(
            &self.conn,
            kb.code,
            window,
            kb.modmask,
        ).request_check().map_err(|_|
            XError::ServerError(
                format!("Unable to ungrab key {} for window {}", 
                kb.code, window)
            )
        )?;
        Ok(())

    }

    fn grab_button(&self, mb: &Mousebind, window: XWindowID, confine: bool) -> Result<()> {
        trace!("Grab button {:?} for window: {}", mb.button, window);

        for m in MODIFIERS.iter() {
            xcb::grab_button(
                &self.conn, 
                false, 
                window, 
                util::ROOT_BUTTON_GRAB_MASK as u16,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
                if confine { window } else { xcb::NONE },
                xcb::NONE,
                mb.button.into(),
                mb.modmask() | m,
            ).request_check().map_err(|_|
                XError::ServerError(
                    format!("Unable to grab button {:?} for window {}", mb.button, window)
                )
            )?;
        }

        Ok(())

    }

    fn ungrab_button(&self, mb: &Mousebind, window: XWindowID) -> Result<()> {
        trace!("Ungrabbing button {:?} for window {}", mb.button, window);

        xcb::ungrab_button(
            &self.conn,
            mb.button.into(),
            window,
            mb.modmask(),
        ).request_check().map_err(|_|
            XError::ServerError(
                format!("Unable to ungrab button {:?} for window {}",
                    mb.button, window
                )
            )
        )
    }

    fn grab_pointer(&self, winid: XWindowID, _mask: u32) -> Result<()> {
        trace!("Grabbing pointer for window: {:?}", winid);

        xcb::grab_pointer(
            &self.conn,
            false,
            winid,
            util::ROOT_POINTER_GRAB_MASK as u16,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::NONE,
            xcb::NONE,
            xcb::CURRENT_TIME,
        ).get_reply()?;

        Ok(())
    }

    fn ungrab_pointer(&self) -> Result<()> {
        trace!("Ungrabbing pointer");

        xcb::ungrab_pointer(&self.conn, xcb::CURRENT_TIME).request_check()?;

        Ok(())
    }

    #[instrument(target="xconn", level="trace", skip(self))]
    fn create_window(&self, ty: WindowClass, geom: Geometry, managed: bool) -> Result<XWindowID> {
        let (ty, bwidth, class, mut data, depth, visualid) = match ty {
            WindowClass::CheckWin => (
                None,
                0,
                xcb::WINDOW_CLASS_INPUT_OUTPUT,
                Vec::new(),
                0,
                0,
            ),
            WindowClass::InputOnly => (
                None,
                0,
                xcb::WINDOW_CLASS_INPUT_ONLY,
                Vec::new(),
                0,
                0,
            ),
            WindowClass::InputOutput(a) => {
                let id = self.conn.generate_id();
                let screen = self.screen(self.idx as usize)?;
                let depth = self.depth(&screen)?;
                let visual = self.visual_type(&depth)?;

                xcb::create_colormap(
                    &self.conn,
                    xcb::COLORMAP_ALLOC_NONE as u8,
                    id,
                    screen.root(),
                    visual.visual_id(),
                ).request_check()?;

                (
                    Some(a),
                    BORDER_WIDTH,
                    xcb::WINDOW_CLASS_INPUT_OUTPUT,
                    vec![
                        (xcb::CW_BORDER_PIXEL, util::FOCUSED_COL),
                        (xcb::CW_COLORMAP, id),
                        (
                            xcb::CW_EVENT_MASK,
                            xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS,
                        ),
                    ],
                    depth.depth(),
                    visual.visual_id(),
                )
            }
        };

        if !managed {
            data.push((xcb::CW_OVERRIDE_REDIRECT, 1));
        }

        let wid = self.conn.generate_id();
        xcb::create_window(
            &self.conn,
            depth,
            wid,
            self.root.id,
            geom.x as i16,
            geom.y as i16,
            geom.width as u16,
            geom.height as u16,
            bwidth as u16,
            class as u16,
            visualid,
            &data,
        ).request_check()?;

        if let Some(a) = ty {
            let net_name = Atom::NetWmWindowType.as_ref();
            self.set_property(
                wid, 
                net_name, 
                Property::Atom(vec![a.as_ref().into()])
            )?;
        }

        if !self.conn.flush() {
            return Err(XError::RequestError("could not flush conn"))
        }

        Ok(wid)
    }

    // Window-related operations
    fn map_window(&self, window: XWindowID) -> Result<()> {
        trace!("Mapping window {}", window);

        let cookie = xcb::map_window(&self.conn, window);
        if let Err(e) = cookie.request_check() {
            error!("Could not map window {}: {}", window, e);
            Err(e.into())
        } else {
            Ok(())
        }
    }

    fn unmap_window(&self, window: XWindowID) -> Result<()> {
        trace!("Unmapping window {}", window);

        let cookie = xcb::unmap_window(&self.conn, window);
        if let Err(e) = cookie.request_check() {
            error!("Could not unmap window {}: {}", window, e);
            Err(e.into())
        } else {
            Ok(())
        }
    }

    fn destroy_window(&self, window: XWindowID) -> Result<()> {
        let atom = Atom::WmDeleteWindow.as_ref();
        let atomval = self.atom(atom)?;
        if self.win_supports(atom, window) {
            trace!("Destroying via ICCCM WM_DELETE_WINDOW");
            let event = ClientMessageEvent {
                window,
                data: ClientMessageData::U32([atomval, 0, 0, 0, 0]),
                type_: atomval,
            };
            return self.send_client_message(window, event)
        } else {
            trace!("Destroying via xcb::destroy_window");
            xcb::destroy_window(&self.conn, window).request_check()?;
        }
        Ok(())
    }

    fn send_client_message(&self, window: XWindowID, data: ClientMessageEvent) -> Result<()> {
        use ClientMessageData::*;

        trace!("Sending client message to window {}", window);

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
        trace!("Setting focus for window {}", window);
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

    fn get_prop(&self, prop: &str, window: XWindowID) -> Result<Option<Property>> {
        let atom = self.atom(prop)?;
        self.get_prop_atom(atom, window)
    }

    fn set_root_scr(&mut self, _scr: i32) {
        todo!()
    }

    fn change_window_attributes(&self, window: XWindowID, attrs: &[ClientAttrs]) -> Result<()> {
        trace!("Changing window attributes");
        for attr in attrs {
            let attr2: Vec<_> = attr.into();
            xcb::change_window_attributes_checked(&self.conn, window, &attr2).request_check()?;
        }
        Ok(())
    }

    fn configure_window(&self, window: XWindowID, attrs: &[ClientConfig]) -> Result<()> {
        trace!("Configuring window {} with attrs {:?}", window, attrs);
        for attr in attrs {
            let attr2: Vec<_> = attr.into();
            xcb::configure_window_checked(&self.conn, window, &attr2).request_check()?
        }
        Ok(())
    }

    fn reparent_window(&self, window: XWindowID, parent: XWindowID) -> Result<()> {
        trace!("Reparenting window {} under parent {}", window, parent);

        Ok(xcb::reparent_window(
            &self.conn,
            window,
            parent,
            0, 0 //placeholder values
        ).request_check()?)
    }
}