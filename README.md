# toaruwm

_A certain X window manager_

A tiling X11 window manager library written in Rust.

It supports multiple backends, such as xcb or X11.

It mainly follows the style of dynamic window managers such as XMonad and QTile, with a main window/region and satellite windows on the side,
but it can also support a wide range of different layouts.

It supports multiple workspaces, and can send windows between all of them. Randr support is planned.
It can also toggle window states between floating and tiling, and preserves this state between desktops.

Non-reparenting (for now, but based on the design goals, it may become a reality).

I do not plan to fully implement ICCCM or EWMH compliance.
See [this](http://www.call-with-current-continuation.org/rants/icccm.txt) for why.

Current SLOC count: `3752`

Heavily inspired by [penrose](https://docs.rs/penrose/0.2.0/penrose/index.html) by sminez.

Design goals:

- Partial ICCCM + EWMH support, just enough to get by.
- Multiple methods of configuration (in order of preference)
  - Lua
  - TOML (?)
  - Directly in the source code (if used as a library)
- IPC via a custom client
- Builtin bars + support for other bars
- Available as a library for users to build custom WMs
  - Can be compiled with or without scripting support (cargo features)
