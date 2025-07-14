# toaruwm

## A certain desktop

---

A desktop creation library written in Rust. Some of its features include:

- Can be downloaded as a binary or used as a crate.
- Supports multiple X backends, including user-implemented ones.
- Multiple methods of configuration, and can be compiled with or without them.

ToaruWM is all about choices. You can add or remove as many features as you like, and configure it and compile it however you like.

It mainly follows the style of dynamic window managers such as XMonad and QTile, with a main window/region and satellite windows on the side,
but it can also support a wide range of different layouts.

It supports multiple workspaces, and can send windows between all of them. Randr is supported insofar as detecting screens and dynamically reconfiguring is concerned.

It can also toggle window states between floating and tiling, and preserves this state between desktops.

Non-reparenting (for now, but based on the design goals, it may become a reality).

I do not plan to fully implement ICCCM or EWMH compliance.
See [this](http://www.call-with-current-continuation.org/rants/icccm.txt) for why.

This crate is NOT production ready; far from it. However, it is under active development and will be ready soon.

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

## Testing

See the [testing](TESTING.md) readme for details.

## Acknowledgements

Special thanks to:

- [penrose](https://docs.rs/penrose/0.2.0/penrose/index.html) by Sminez, for inspiring this crate.
- [smithay](https://github.com/Smithay/smithay) and [x11rb](https://github.com/psychon/x11rb), for powering this crate.
- [niri](https://github.com/YaLTeR/niri), for showing how to do certain things.
