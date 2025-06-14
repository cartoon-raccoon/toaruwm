#![allow(unused_imports, dead_code)]

use std::collections::{HashMap, VecDeque};

use crate::core::{Client, Screen};
use crate::types::Geometry;
use crate::platform::x::{
    core::{PointerQueryReply, Result, XAtom, XConn, XError, XWindow, XWindowID},
    event::XEvent,
};

/// A dummy connection implementing XConn but actually
/// does not interface with the X server at all, and
/// should mainly be used for testing.
///
/// `DummyConn` contains an internal queue that takes
/// XEvents and dequeues them when `poll_next_event` is
/// called, as well as an internal store of windows,
/// representing top-level windows managed by the
/// window manager.
pub struct DummyConn {
    events: VecDeque<XEvent>,
    root: XWindow,
    children: HashMap<XWindowID, Client>,
}

impl DummyConn {
    /// Creates a new DummyConn.
    pub fn new(root: XWindow) -> Self {
        Self {
            events: VecDeque::new(),
            root,
            children: HashMap::new(),
        }
    }

    /// Adds a single event to the internal queue to be sent out by
    /// `XConn::poll_next_event`.
    pub fn add_event(&mut self, event: XEvent) {
        self.events.push_back(event);
    }

    /// Adds multiple events to the internal queue.
    pub fn add_events<I>(&mut self, events: I)
    where
        I: IntoIterator<Item = XEvent>,
    {
        for event in events {
            self.events.push_back(event);
        }
    }

    pub fn add_window(&mut self, window: Client) {
        self.children.insert(window.id(), window);
    }

    pub fn replace_root(&mut self, root: XWindow) {
        self.root = root
    }
}

// todo
// impl XConn for DummyConn {
//     fn poll_next_event(&self) -> Result<Option<XEvent>> {
//         Ok(self.events.pop_front())
//     }

//     fn get_root(&self) -> XWindow {
//         self.root
//     }

//     fn get_geometry(&self, window: XWindowID) -> Result<Geometry> {
//         Ok(self.children.get(&window).ok_or(
//             XError::OtherError(format!("no such window {}", window))
//         )?.geometry())
//     }

//     fn query_tree(&self, window: XWindowID) -> Result<Vec<XWindowID>> {
//         Ok(self.children.values().map(|c| c.id()).collect())
//     }

//     fn query_pointer(&self, window: XWindowID) -> Result<PointerQueryReply> {
//         todo!()
//     }

//     fn all_outputs(&self) -> Result<Vec<Screen>> {
//         todo!()
//     }

//     fn atom(&self, atom: &str) -> Result<XAtom> {
//         todo!()
//     }
// }
