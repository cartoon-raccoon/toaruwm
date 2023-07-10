use xcb::x::EventMask;

// Root window mouse button event mask
pub const ROOT_BUTTON_GRAB_MASK: EventMask = 
    EventMask::BUTTON_PRESS.union(EventMask::BUTTON_RELEASE);

// Root window pointer event mask
pub const ROOT_POINTER_GRAB_MASK: EventMask = 
    EventMask::BUTTON_RELEASE.union(EventMask::BUTTON_MOTION);

/// A macro for creating `xcb::XidNew` objects from a `u32`.
macro_rules! cast {
    ($ctype:ty, $resid:expr) => {
        unsafe {<$ctype as XidNew>::new($resid)}
    };
}

/// A macro for extracting id from objects implementing `x::Xid`.
macro_rules! id {
    ($e:expr) => {
        $e.resource_id()
    }
}

/// A macro for the common pattern off sending a request
/// and then getting the reply from the cookie that gets returned.
/// 
/// Note that this completely disregards the asynchronous
/// nature of the underlying XCB library.
macro_rules! req_and_reply {
    ($conn:expr, $req:expr) => {
        $conn.wait_for_reply($conn.send_request($req))
    }
}

/// A macro for the common pattern off sending a request
/// and then getting a result from the cookie that gets returned.
/// 
/// Note that this completely disregards the asynchronous
/// nature of the underlying XCB library.
macro_rules! req_and_check {
    ($conn:expr, $req:expr) => {
        $conn.check_request($conn.send_request_checked($req))
    }
}

pub(super) use {
    cast, id, 
    req_and_reply,
    req_and_check,
};