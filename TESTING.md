# Testing

ToaruWM is designed for use as a crate in your own projects, but
it can also be compiled and run as an executable. Currently, the
main entry point for the testing executable is located
[here](src/bin/main.rs), and you can use it as a basic example as
to how it should be used as a library.

You can test this repository on your own system via Xephyr, which
is a program that allows you to run a separate instance of the X server
within a pre-existing X session in a separate window.

To run this repository, you need to have Rust installed on your system.
You can install Cargo from your distro's repos, or from the official
Rust [website](https://www.rust-lang.org/tools/install).

To check out this repository, clone it, change into it and run the following commands:

```shell
Xephyr -br -ac -noreset -screen <resolution> :1 &

DISPLAY=:1 cargo run
```

where `<resolution>` is the resolution you want for the window,
e.g. 1500x600.

By default, three workspaces are provided, creatively named 1, 2, and 3.

The current keybinds are defined in `src/bin/main.rs` as:

```rust
// defining keybinds and associated WM actions
const KEYBINDS: &[(&str, fn(Wm))] = &[
    ("M-Return", |wm| wm.run_external("alacritty", &[])),
    ("M-r",      |wm| wm.run_external("dmenu_run", &["-b"])),
    ("M-q",      |wm| wm.close_focused_window()),
    ("M-S-d",    |wm| wm.dump_internal_state()),
    ("M-S-q",    |wm| wm.quit()),
    ("M-S-Up",   |wm| wm.warp_window(5, Up)),
    ("M-S-Down", |wm| wm.warp_window(5, Down)),
    ("M-S-Left", |wm| wm.warp_window(5, Left)),
    ("M-S-Right",|wm| wm.warp_window(5, Right)),
    ("M-1",      |wm| wm.goto_workspace("1")),
    ("M-2",      |wm| wm.goto_workspace("2")),
    ("M-3",      |wm| wm.goto_workspace("3")),
    ("M-S-1",    |wm| wm.send_focused_to("1")),
    ("M-S-2",    |wm| wm.send_focused_to("2")),
    ("M-S-3",    |wm| wm.send_focused_to("3")),
];
```

Which translate to:

- `Mod+Enter`: Run terminal program (in this case alacritty)
- `Mod+r`: Run program launcher (in this case dmenu)
- `Mod+q`: Close the focused window
- `Mod+Shift+d`: Dumps the internal state of the WM. Used for testing only.
- `Mod+Shift+q`: Quits the WM.
- `Mod+Shift+{Up,Down,Left,Right}`: Move the focused window 5 pixels in the given direction.
- `Mod+{1,2,3}`: Go to workspace 1, 2, or 3.
- `Mod+Shift+{1,2,3}`: Send the focused window to workspace 1, 2, or 3.

The mousebindings are:

- `Mod + Left`: Move window
- `Mod + Right`: Resize window

The `Mod` button in this case is the Meta (windows) key. To use the alt key, replace the `M` in the keybind string with `A`.

## Bugs

Currently, some keybinds are buggy. These include:

- `warp_window`. This unsets the master window of the workspace when tiled.
- `send_focused_to`. This fails the set the focused window after sending the window.
