# xrf-game-platform

The game's window and input, over [SDL 3](https://libsdl.org/). It is the only crate that names SDL: the rest of the
game reads `PlatformEvent`s and tells the window what to do with `PlatformWindowCommand`s.

## Starting

`Platform::start()` loads SDL's library, starts its video and gamepads, and must run on the main thread, which SDL
takes as its own. `open_window(&PlatformWindowRequest)` opens the window, resizable and centred, windowed or
fullscreen. Both stay on that thread.

On Windows every binary delay-loads `SDL3.dll` (`.cargo/config.toml`), so a binary that never starts the platform runs
without it. `start` loads it by its full path first: beside the executable, as a packaged game ships it, else from the
checkout's `.sdks/sdl3`, which `cargo make fetch-sdks` fills. The build script points the linker at `SDL3.lib` through
`SDL3_LIB_DIR`. SDL is fetched for Windows only, so the Linux CI job leaves the `xrf-game` crates out.

## Events

`Platform::wait(timeout, &mut events)` waits for an event until the timeout has passed and appends every waiting
event, so the main thread rests until there is input or its own next work is due; a zero timeout only reads. SDL waits
whole milliseconds and a millisecond short of what it is asked (asked one, it returns at once), so the platform asks
it for one more: a wait ends up to about a millisecond late, never early (asked 4 ms, 4.4 ms at the median and 5.5 at
p99 on Windows).

Another thread ends the wait through a `PlatformWaker`, which pushes a user event the wait returns on and appends
nothing for. It may be made before the platform starts and handed to threads that start first; `Platform::attach`
takes it, a wake asked before then ends the first wait, and wakes between two waits count as one. `to_waker` gives the
standard library's `Waker` for crates that know nothing of the platform. Each event is stamped with when it happened on SDL's clock (`PlatformEvent::at`):

| Kind                        | What it carries                                                                      |
| --------------------------- | ------------------------------------------------------------------------------------ |
| `Quit`                      | The close button or the system asked to quit                                         |
| `Resized`                   | The new drawable size, in device pixels                                              |
| `Shown`, `Focused`          | Minimised or shown again; focus gained or lost                                       |
| `Key`                       | A physical key as a `PlatformScancode` (SDL's and USB HID's numbers), down, repeat   |
| `Text`                      | Typed text, while the window accepts it                                              |
| `MouseMoved`, `MouseButton` | The position in logical units, and the device's own motion                           |
| `MouseWheel`                | Notches, up and right positive, a flipped wheel turned back                          |
| `Gamepad*`                  | A pad connected or removed, its buttons and axes by SDL's numbers, axes from -1 to 1 |

A gamepad is opened as it connects, which is what makes SDL report its buttons. Events the game does not act on are
dropped.

## The window

- `PlatformWindow::create_hosted()` answers the window as the renderer's viewports attach to it: an
  `xrf-renderer-host` `HostedWindow` over a clone of SDL's window. Drop it on the main thread once the detach of its
  viewports is handled.
- `get_size()` is the drawable size in device pixels and `get_scale()` the display's scale.
- `apply(PlatformWindowCommand)` sets the mode or the windowed size, grabs the cursor (hidden, kept in the window,
  motion reported raw) and starts or stops accepting text.

A window started from a background process, as an agent's bridge starts the game, is not given the foreground by
Windows: it has no keyboard focus until clicked.

## Checks

Run `cargo test --locked -p xrf-game-platform` from the repository root. The tests read SDL's events into the game's
and call nothing in SDL's library; opening a window is checked by running the game.
