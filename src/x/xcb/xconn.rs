//! Implementation of the XConn trait for XCBConn.

use xcb::randr;
use xcb::x;
use xcb::{Xid, XidNew};

use tracing::instrument;
use tracing::{error, trace, warn};

use super::Initialized;
use super::{cast, id, req_and_check, req_and_reply, util};
use crate::bindings::{Keybind, Mousebind};
use crate::core::Screen;
use crate::types::{ClientAttrs, ClientConfig, Geometry};
use crate::x::{
    core::{PointerQueryReply, Result, WindowClass, XAtom, XConn, XError, XWindow, XWindowID},
    event::{ClientMessageData, ClientMessageEvent, XEvent},
    input::MODIFIERS,
    property::*,
    Atom,
};

use super::XCBConn;

impl XConn for XCBConn<Initialized> {
    // General X server operations
    #[instrument(target = "xconn", level = "trace", skip(self))]
    fn poll_next_event(&self) -> Result<Option<XEvent>> {
        self.conn.flush()?;

        let event = self.conn.wait_for_event()?;
        Ok(Some(self.process_raw_event(event)?))
    }

    fn get_root(&self) -> XWindow {
        self.root
    }

    fn get_geometry(&self, window: XWindowID) -> Result<Geometry> {
        self.get_geometry_inner(window)
    }

    fn query_tree(&self, window: XWindowID) -> Result<Vec<XWindowID>> {
        trace!("Querying tree");
        let res: x::QueryTreeReply = req_and_reply!(
            self.conn,
            &x::QueryTree {
                window: cast!(x::Window, window)
            } // get the reply and map its ok to grab its children
        )?;
        let ret = res.children().iter().map(|child| id!(child)).collect();
        Ok(ret)
    }

    fn query_pointer(&self, window: XWindowID) -> Result<PointerQueryReply> {
        let reply = req_and_reply!(
            &self.conn,
            &x::QueryPointer {
                window: cast!(x::Window, window)
            }
        )?;

        Ok(PointerQueryReply {
            same_screen: reply.same_screen(),
            root: id!(reply.root()),
            child: id!(reply.child()),
            root_x: reply.root_x() as i32,
            root_y: reply.root_x() as i32,
            win_x: reply.win_x() as i32,
            win_y: reply.win_y() as i32,
            mask: reply.mask().into(),
        })
    }

    #[instrument(target = "xconn", level = "trace", skip(self))]
    fn all_outputs(&self) -> Result<Vec<Screen>> {
        let check_id = self.check_win()?;
        self.conn.flush()?;

        let res = self
            .conn
            .wait_for_reply(self.conn.send_request(&randr::GetScreenResources {
                window: cast!(x::Window, check_id),
            }))?;

        let info = req_and_reply!(
            self.conn,
            &randr::GetScreenInfo {
                window: cast!(x::Window, check_id)
            }
        )?;

        let crtcs = res
            .crtcs()
            .iter()
            // could do this with flat_map, but that just seems confusing
            // for each crtc, get its info
            .map(|c| {
                self.conn
                    .wait_for_reply(self.conn.send_request(&randr::GetCrtcInfo {
                        crtc: *c,
                        config_timestamp: 0,
                    }))
            })
            // filter out errors
            // todo: add a warning?
            .filter_map(|r| r.ok())
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
                Screen::new(i as i32, geom, id!(info.root()), vec![])
            })
            .filter(|s| s.true_geom().width > 0)
            .collect();

        req_and_check!(
            self.conn,
            &x::DestroyWindow {
                window: cast!(x::Window, check_id)
            }
        )?;

        Ok(crtcs)
    }

    fn atom(&self, atom: &str) -> Result<XAtom> {
        if let Some(known) = self.atoms().retrieve(atom) {
            return Ok(known);
        }
        trace!("Interning atom {}", atom);
        let x = req_and_reply!(
            self.conn,
            &x::InternAtom {
                only_if_exists: false,
                name: atom.as_bytes()
            }
        )?;
        trace!("Atom name: {}, atom: {}", atom, id!(x.atom()));
        Ok(id!(x.atom()))
    }

    fn lookup_atom(&self, atom: XAtom) -> Result<String> {
        trace!("Looking up atom {}", atom);
        if let Some(name) = self.atoms().retrieve_by_value(atom) {
            trace!("Got name {}", name);
            return Ok(name);
        }
        trace!("Name not known, looking up via X connection");
        let name = req_and_reply!(
            self.conn,
            &x::GetAtomName {
                atom: cast!(x::Atom, atom)
            }
        )?
        .name()
        .to_string();

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
        let _ = req_and_reply!(
            &self.conn,
            &x::GrabKeyboard {
                owner_events: false,
                grab_window: x::Window::none(),
                time: x::CURRENT_TIME,
                pointer_mode: x::GrabMode::Async,
                keyboard_mode: x::GrabMode::Async,
            }
        )
        .map_err(|_| XError::ServerError("Unable to grab keyboard".into()))?
        .status();
        //todo: handle the status
        Ok(())
    }

    fn ungrab_keyboard(&self) -> Result<()> {
        trace!("Ungrabbing kayboard");

        req_and_check!(
            self.conn,
            &x::UngrabKeyboard {
                time: x::CURRENT_TIME,
            }
        )
        .map_err(|_| XError::ServerError("Unable to ungrab keyboard".into()))?;

        Ok(())
    }

    fn grab_key(&self, kb: Keybind, window: XWindowID) -> Result<()> {
        trace!("Grabbing key {} for window {}", kb.code, window);

        for m in MODIFIERS.iter() {
            req_and_check!(
                self.conn,
                &x::GrabKey {
                    owner_events: false,
                    grab_window: cast!(x::Window, window),
                    modifiers: (kb.modmask | *m).into(),
                    key: kb.code,
                    pointer_mode: x::GrabMode::Async,
                    keyboard_mode: x::GrabMode::Async,
                }
            )
            .map_err(|_| {
                XError::ServerError(format!(
                    "Unable to grab key {} for window {}",
                    kb.code, window
                ))
            })?;
        }

        Ok(())
    }

    fn ungrab_key(&self, kb: Keybind, window: XWindowID) -> Result<()> {
        //let code = KeySymbols::new(&self.conn).get_keycode(kb.keysym).next();

        req_and_check!(
            self.conn,
            &x::UngrabKey {
                key: kb.code,
                grab_window: cast!(x::Window, window),
                modifiers: x::ModMask::from(kb.modmask),
            }
        )
        .map_err(|_| {
            XError::ServerError(format!(
                "Unable to ungrab key {} for window {}",
                kb.code, window
            ))
        })?;
        Ok(())
    }

    fn grab_button(&self, mb: Mousebind, window: XWindowID, confine: bool) -> Result<()> {
        trace!("Grab button {:?} for window: {}", mb.button, window);

        for m in MODIFIERS.iter() {
            req_and_check!(
                self.conn,
                &x::GrabButton {
                    owner_events: false,
                    grab_window: cast!(x::Window, window),
                    event_mask: util::ROOT_BUTTON_GRAB_MASK,
                    pointer_mode: x::GrabMode::Async,
                    keyboard_mode: x::GrabMode::Async,
                    confine_to: if confine {
                        cast!(x::Window, window)
                    } else {
                        x::Window::none()
                    },
                    cursor: x::Cursor::none(),
                    button: mb.button.into(),
                    modifiers: (mb.modmask | *m).into(),
                }
            )
            .map_err(|_| {
                XError::ServerError(format!(
                    "Unable to grab button {:?} for window {}",
                    mb.button, window
                ))
            })?;
        }

        Ok(())
    }

    fn ungrab_button(&self, mb: Mousebind, window: XWindowID) -> Result<()> {
        trace!("Ungrabbing button {:?} for window {}", mb.button, window);

        req_and_check!(
            self.conn,
            &x::UngrabButton {
                button: mb.button.into(),
                grab_window: cast!(x::Window, window),
                modifiers: mb.modmask(),
            }
        )
        .map_err(|_| {
            XError::ServerError(format!(
                "Unable to ungrab button {:?} for window {}",
                mb.button, window
            ))
        })
    }

    fn grab_pointer(&self, winid: XWindowID, _mask: u32) -> Result<()> {
        trace!("Grabbing pointer for window: {:?}", winid);

        let _ = req_and_reply!(
            self.conn,
            &x::GrabPointer {
                owner_events: false,
                grab_window: cast!(x::Window, winid),
                event_mask: util::ROOT_POINTER_GRAB_MASK,
                pointer_mode: x::GrabMode::Async,
                keyboard_mode: x::GrabMode::Async,
                confine_to: x::Window::none(),
                cursor: x::Cursor::none(),
                time: x::CURRENT_TIME,
            }
        )?;

        Ok(())
    }

    fn ungrab_pointer(&self) -> Result<()> {
        trace!("Ungrabbing pointer");

        req_and_check!(
            self.conn,
            &x::UngrabPointer {
                time: x::CURRENT_TIME
            }
        )?;

        Ok(())
    }

    #[instrument(target = "xconn", level = "trace", skip(self))]
    fn create_window(&self, ty: WindowClass, geom: Geometry, managed: bool) -> Result<XWindowID> {
        let (ty, bwidth, class, mut data, depth, visualid) = match ty {
            WindowClass::CheckWin => (
                None,
                0,
                x::WindowClass::InputOutput,
                Vec::<x::Cw>::new(),
                0,
                0,
            ),
            WindowClass::InputOnly => (
                None,
                0,
                x::WindowClass::InputOnly,
                Vec::<x::Cw>::new(),
                0,
                0,
            ),
            WindowClass::InputOutput(a, b) => {
                let mid: x::Colormap = self.conn.generate_id();
                let screen = self.screen(self.idx as usize)?;
                let depth = self.depth(screen)?;
                let visual = self.visual_type(depth)?;

                req_and_check!(
                    self.conn,
                    &x::CreateColormap {
                        alloc: x::ColormapAlloc::None,
                        mid,
                        window: screen.root(),
                        visual: visual.visual_id(),
                    }
                )?;

                (
                    Some(a),
                    b,
                    x::WindowClass::InputOutput,
                    vec![
                        x::Cw::BorderPixel(0x00000000), //fixme: see above
                        x::Cw::Colormap(mid),
                        x::Cw::EventMask(x::EventMask::EXPOSURE | x::EventMask::KEY_PRESS),
                    ],
                    depth.depth(),
                    visual.visual_id(),
                )
            }
        };

        if !managed {
            data.push(x::Cw::OverrideRedirect(true));
        }
        let wid: x::Window = self.conn.generate_id();
        req_and_check!(
            self.conn,
            &x::CreateWindow {
                depth: depth,
                wid,
                parent: cast!(x::Window, self.root.id),
                x: geom.x as i16,
                y: geom.y as i16,
                width: geom.width as u16,
                height: geom.height as u16,
                border_width: bwidth as u16,
                class,
                visual: visualid,
                value_list: &data,
            }
        )?;

        if let Some(a) = ty {
            let net_name = Atom::NetWmWindowType.as_ref();
            self.set_property(id!(wid), net_name, Property::Atom(vec![a.as_ref().into()]))?;
        }

        self.conn.flush()?;

        Ok(id!(wid))
    }

    // Window-related operations
    fn map_window(&self, window: XWindowID) -> Result<()> {
        trace!("Mapping window {}", window);

        let cookie = req_and_check!(
            self.conn,
            &x::MapWindow {
                window: cast!(x::Window, window)
            }
        );
        if let Err(e) = cookie {
            error!("Could not map window {}: {}", window, e);
            Err(e.into())
        } else {
            Ok(())
        }
    }

    fn unmap_window(&self, window: XWindowID) -> Result<()> {
        trace!("Unmapping window {}", window);

        let cookie = req_and_check!(
            self.conn,
            &x::UnmapWindow {
                window: cast!(x::Window, window)
            }
        );
        if let Err(e) = cookie {
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
            return self.send_client_message(window, event);
        } else {
            trace!("Destroying via the destroy window request");
            req_and_check!(
                self.conn,
                &x::DestroyWindow {
                    window: cast!(x::Window, window)
                }
            )?;
        }
        Ok(())
    }

    fn send_client_message(&self, window: XWindowID, data: ClientMessageEvent) -> Result<()> {
        use ClientMessageData::*;

        trace!("Sending client message to window {}", window);

        let to_send = match data.data {
            U8(bytes) => x::ClientMessageData::Data8(bytes),
            U16(words) => x::ClientMessageData::Data16(words),
            U32(dwords) => x::ClientMessageData::Data32(dwords),
        };

        let event = x::ClientMessageEvent::new(
            cast!(x::Window, window),
            cast!(x::Atom, data.type_),
            to_send,
        );

        Ok(req_and_check!(
            self.conn,
            &x::SendEvent {
                propagate: false,
                destination: x::SendEventDest::Window(cast!(x::Window, window)),
                event_mask: x::EventMask::NO_EVENT,
                event: &event,
            }
        )?)
    }

    #[allow(unused_must_use)]
    fn set_input_focus(&self, window: XWindowID) {
        trace!("Setting focus for window {}", window);
        req_and_check!(
            self.conn,
            &x::SetInputFocus {
                revert_to: x::InputFocus::PointerRoot,
                focus: cast!(x::Window, window),
                time: x::CURRENT_TIME
            }
        ); //* FIXME: use the error
    }

    fn set_geometry(&self, window: XWindowID, geom: Geometry) -> Result<()> {
        self.configure_window(
            window,
            &[ClientConfig::Resize {
                h: geom.height,
                w: geom.width,
            }],
        )?;

        self.configure_window(
            window,
            &[ClientConfig::Move {
                x: geom.x,
                y: geom.y,
            }],
        )?;

        Ok(())
    }

    fn set_property(&self, window: XWindowID, prop: &str, data: Property) -> Result<()> {
        use Property::*;

        let mode = x::PropMode::Replace;
        let atom = self.atom(prop)?;

        let (ty, data) = match data {
            Atom(atoms) => (
                x::ATOM_ATOM,
                atoms.iter().map(|a| self.atom(a).unwrap_or(0)).collect(),
            ),
            Cardinal(card) => (x::ATOM_CARDINAL, vec![card]),
            String(strs) | UTF8String(strs) => {
                return Ok(req_and_check!(
                    self.conn,
                    &x::ChangeProperty {
                        mode,
                        window: cast!(x::Window, window),
                        property: cast!(x::Atom, atom),
                        r#type: x::ATOM_STRING,
                        data: strs.join("\0").as_bytes()
                    }
                )?)
            }
            Window(ids) => (x::ATOM_WINDOW, ids),
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

        Ok(req_and_check!(
            self.conn,
            &x::ChangeProperty {
                mode,
                window: cast!(x::Window, window),
                property: cast!(x::Atom, atom),
                r#type: ty,
                data: &data
            }
        )?)
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
        let attrs: Vec<x::Cw> = attrs.iter().map(|i| i.into()).collect();

        req_and_check!(
            self.conn,
            &x::ChangeWindowAttributes {
                window: cast!(x::Window, window),
                value_list: &attrs
            }
        )?;
        Ok(())
    }

    fn configure_window(&self, window: XWindowID, attrs: &[ClientConfig]) -> Result<()> {
        trace!("Configuring window {} with attrs {:?}", window, attrs);
        for attr in attrs {
            let mut attr2 = Vec::from(attr);
            attr2.sort();
            trace!("{:?}", attr2);
            req_and_check!(
                self.conn,
                &x::ConfigureWindow {
                    window: cast!(x::Window, window),
                    value_list: &attr2
                }
            )?
        }
        Ok(())
    }

    fn reparent_window(&self, window: XWindowID, parent: XWindowID) -> Result<()> {
        trace!("Reparenting window {} under parent {}", window, parent);

        Ok(req_and_check!(
            self.conn,
            &x::ReparentWindow {
                window: cast!(x::Window, window),
                parent: cast!(x::Window, parent),
                x: 0,
                y: 0 //* FIXME: placeholder values */ */
            }
        )?)
    }
}
