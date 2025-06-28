# Compliance

This document tracks the compliance of Toaru to various Wayland protocols and X11 specifications.

## Notation

Item Support Labels are listed as they are intended for the _final state_ to be. If an item is not listed,
no assumptions should be made about its future implementation, even though it might be implemented in
the future.

- **Fully Supported:** Support for the item is fully implemented.
- **Partially Supported:** Support for the item is partially implemented.
- **Not Supported:** Support for the item is intentionally not implemented, and no assumptions should be made
about its implementation in the future.

### Modifiers

- \[todo\]: Implementation is planned, but has not started. It can however, be assumed to be completed
at some point in the future.
- \[IP\]: Implementation has started, but is not complete. It can be assumed to be completed at some
point in the future.
- Remarks are enclosed in parentheses `()`.

## Wayland Compliance

There are no partial implementations with respect to Wayland protocols.

**Fully Supported:**

- `wl_display` \[IP\](Wayland core)
- `wp_presentation` \[todo\] (Presentation time)
- `wp_viewporter` \[todo\] (Viewporter)
- `xdg_wm_base` \[IP\] (XDG Shell)
- `zwp_linux_dmabuf_v1` \[todo\] (Linux DMA-BUF)
- `xdg_activation_v1` \[todo\] (XDG activation)
- `wp_drm_lease_device_v1` \[todo\] (DRM lease)
- `wp_content_type_manager_v1` \[todo\] (Content type hint)
- `wp_fractional_scale_manager_v1` \[todo\] (Fractional scale)
- `zxdg_decoration_manager_v1` \[IP\] (XDG decoration)

**Not Supported:**

- `ext_image_capture_source_v1` (Image capture source)
- `xdg_system_bell_v1` (XDG system bell)

## X11 Compliance

### ICCCM Compliance

**Fully Supported:**

- `WM_NAME`
- `WM_ICON_NAME`
- `WM_CLASS`
- `WM_TRANSIENT_FOR`

**Partially Supported:**

- `WM_PROTOCOLS`
  - `WM_DELETE_WINDOW`
- `WM_NORMAL_HINTS`
  - `min_aspect`, `max_aspect`
  - `base`
  - `gravity`
- `WM_STATE`

**Not Supported:**

_Anything from the ICCCM spec that is not listed above is not supported._

### EWMH Compliance

#### Root Window Properties

**Fully Supported:**

- `_NET_SUPPORTED`
- `_NET_NUMBER_OF_DESKTOPS`
- `_NET_CURRENT_DESKTOP`
- `_NET_WORKAREA` \[todo\]
- `_NET_CLIENT_LIST` \[todo\]
- `_NET_DESKTOP_GEOMETRY` \[todo\]
- `_NET_CURRENT_DESKTOP` \[todo\]
- `_NET_DESKTOP_NAMES` \[todo\]
- `_NET_ACTIVE_WINDOW` \[todo\]
- `_NET_DESKTOP_VIEWPORT` \[todo\]

**Partially Supported:**

Nil

**Not Supported:**

Nil

#### Application Window Properties

**Fully Supported:**

- `_NET_WM_NAME`
- `_NET_WM_ICON_NAME`
- `_NET_WM_WINDOW_TYPE` \[todo\]
- `_NET_WM_STATE` \[todo\]
- `_NET_WM_ALLOWED_ACTIONS` \[todo\]
- `_NET_WM_STRUT{,_PARTIAL}` \[todo\]

**Partially Supported:**

Nil

**Not Supported:**

- `_NET_WM_DESKTOP` (might support)
- `_NET_WM_ICON_GEOMETRY`
