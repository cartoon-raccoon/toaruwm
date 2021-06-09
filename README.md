# toaruwm

### A certain X window manager

---

A tiling X11 window manager library written in Rust.

It supports multiple backends, such as xcb or X11.

It mainly follows the style of dynamic window managers such as XMonad and QTile, with a main window/region and satellite windows on the side,
but it can also support a wide range of different layouts.

It supports multiple workspaces, and can send windows between all of them. Randr is partially supported.
It can also toggle window states between floating and tiling, and preserves this state between desktops.

Non-reparenting (for now, but based on the design goals, it may become a reality).

I do not plan to fully implement ICCCM or EWMH compliance.
See [this](http://www.call-with-current-continuation.org/rants/icccm.txt) for why.

Current SLOC count: `4017`

Heavily inspired by [penrose](https://docs.rs/penrose/0.2.0/penrose/index.html) by sminez, approximately 60% of this codebase is based off his code, so a million thanks, and I am forever grateful.

## Design Goals

- Partial ICCCM + EWMH support, just enough to get by.
- Multiple methods of configuration (in order of preference)
  - Custom scripting language
  - Lua
  - Custom config file (disadvantage: not turing-complete)
  - Directly in the source code (if used as a library)
- IPC via a custom client/protocol (maybe?)
- Builtin bars + support for other bars
- Available as a library for users to build custom WMs
  - Can be compiled without some features (cargo features)
- Recreation of layouts from serialized data (JSON most likely)
