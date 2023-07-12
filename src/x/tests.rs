use super::{x11rb::X11RBConn, xcb::XCBConn, Property, XConn, XWindowID};
use crate::x::Atom::*;

#[cfg(debug_assertions)]
macro_rules! debug {
    ($fmt:expr) => {
        (println!(concat!("[debug] ", $fmt)));
    };
    ($fmt:expr, $($arg:tt)*) => {
        (println!(concat!("[debug] ", $fmt), $($arg)*));
    };
}

fn test_property_retrieval_generic<X: XConn>(conn: &X) {
    let err =
        |xid: u32| -> std::string::String { format!("failed to get prop for window {}", xid) };

    let windows = conn.query_tree(conn.get_root().id).unwrap();

    for xid in windows {
        debug!("querying window {}", xid);
        let wm_name = conn.get_property(WmName.as_ref(), xid).expect(&err(xid));
        prop_check_stub(wm_name, is_string, "WM_NAME", xid);

        let net_wm_name = conn.get_property(NetWmName.as_ref(), xid).expect(&err(xid));
        prop_check_stub(net_wm_name, is_string, "NET_WM_NAME", xid);

        let wm_class = conn.get_property(WmClass.as_ref(), xid).expect(&err(xid));
        prop_check_stub(wm_class, is_string, "WM_CLASS", xid);

        let wm_hints = conn.get_property(WmHints.as_ref(), xid).expect(&err(xid));
        prop_check_stub(wm_hints, |p| p.is_wmhints(), "WM_HINTS", xid);

        let wm_size_hints = conn.get_property(WmNormalHints.as_ref(), xid).expect(&err(xid));
        prop_check_stub(wm_size_hints, |p| p.is_sizehints(), "WM_NORMAL_HINTS", xid);
    }
}

#[test]
fn test_property_retrieval_xcb() {
    let mut conn = XCBConn::connect().unwrap();
    conn.init().unwrap();

    test_property_retrieval_generic(&conn);
}

#[test]
fn test_property_retrieval_x11rb() {
    let mut conn = X11RBConn::connect().unwrap();
    conn.init().unwrap();

    test_property_retrieval_generic(&conn);
}

fn prop_check_stub<F: Fn(&Property) -> bool>(
    prop: Option<Property>,
    typecheck: F,
    name: &str,
    id: XWindowID,
) {
    if let Some(prop) = prop {
        // main assertion occurs here
        assert!(typecheck(&prop));
        debug!("{} is {}", name, prop);
    } else {
        debug!("{} not set for window {}", name, id);
    }
}

fn is_string(prop: &Property) -> bool {
    use Property as Prop;

    match prop {
        Prop::String(_) | Prop::UTF8String(_) => true,
        Prop::U8List(s, _) | Prop::U16List(s, _) | Prop::U32List(s, _) if s == "COMPOUND_TEXT" => {
            true
        }
        _ => false,
    }
}

#[test]
fn test_connection_init_xcb() {
    let mut conn = XCBConn::new().unwrap();
    conn.init().expect("could not initialize xcb connection");
}

#[test]
fn test_connection_init_x11rb() {
    let mut conn = X11RBConn::new().unwrap();
    conn.init().expect("could not initialize x11rb connection");
}
