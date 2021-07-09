use super::{
    XCBConn, XConn, XWindowID,
    Property,
};
use crate::x::Atom::*;

#[test]
fn test_ewmh_atoms_values() {
    let mut conn = XCBConn::connect().expect("connection failed");
    conn.init().unwrap();

    let econn = conn.ewmh_conn();

    assert_eq!(
        conn.lookup_interned_atom(NetSupported.as_ref()).unwrap(), 
        econn.SUPPORTED()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetClientList.as_ref()).unwrap(), 
        econn.CLIENT_LIST()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetClientListStacking.as_ref()).unwrap(), 
        econn.CLIENT_LIST_STACKING()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetCurrentDesktop.as_ref()).unwrap(), 
        econn.CURRENT_DESKTOP()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetDesktopNames.as_ref()).unwrap(), 
        econn.DESKTOP_NAMES()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetNumberOfDesktops.as_ref()).unwrap(), 
        econn.NUMBER_OF_DESKTOPS()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeDesktop.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_DESKTOP()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeDock.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_DOCK()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeToolbar.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_TOOLBAR()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeMenu.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_MENU()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeUtility.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_UTILITY()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeSplash.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_SPLASH()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeDialog.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_DIALOG()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeDropdownMenu.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_DROPDOWN_MENU()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypePopupMenu.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_POPUP_MENU()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeNotification.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_NOTIFICATION()
    );
    assert_eq!(
        conn.lookup_interned_atom(NetWindowTypeNormal.as_ref()).unwrap(), 
        econn.WM_WINDOW_TYPE_NORMAL()
    );
}

#[test]
fn test_property_retrieval() {
    let err = |xid: u32| -> std::string::String {
        format!("failed to get prop for window {}", xid)
    };

    let mut conn = XCBConn::connect().unwrap();
    conn.init().unwrap();

    let windows = conn.query_tree(conn.get_root().id).unwrap();

    for xid in windows {
        debug!("querying window {}", xid);
        let wm_name = conn.get_prop(WmName.as_ref(), xid).expect(&err(xid));
        prop_check_stub(wm_name, is_string, "WM_NAME", xid);

        let net_wm_name = conn.get_prop(NetWmName.as_ref(), xid).expect(&err(xid));
        prop_check_stub(net_wm_name, is_string, "NET_WM_NAME", xid);

        let wm_class = conn.get_prop(WmClass.as_ref(), xid).expect(&err(xid));
        prop_check_stub(wm_class, is_string, "WM_CLASS", xid);

        let wm_hints = conn.get_prop(WmHints.as_ref(), xid).expect(&err(xid));
        prop_check_stub(wm_hints, |p| p.is_wmhints(), "WM_HINTS", xid);

        let wm_size_hints = conn.get_prop(WmNormalHints.as_ref(), xid).expect(&err(xid));
        prop_check_stub(wm_size_hints, |p| p.is_sizehints(), "WM_NORMAL_HINTS", xid);
    }
}

fn prop_check_stub<F: Fn(&Property) -> bool>(
    prop: Option<Property>, 
    typecheck: F, 
    name: &str, 
    id: XWindowID
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
        Prop::U8List(s, _) | Prop::U16List(s, _) | Prop::U32List(s, _)
        if s == "COMPOUND_TEXT" => true,
        _ => false
    }
}
