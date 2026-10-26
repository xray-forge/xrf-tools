# xrf-app

The XRF desktop application: the Tauri binary that assembles the plugin crates, opens the main window and runs until it
exits. It holds no domain of its own. Each domain is a plugin crate under `plugins/`, the infrastructure they share is
`xrf-app-core`, and the page the window shows is built from `ui/` (`xrf-app-ui`).

## Layout

- `application`: the composition root. It starts the log and writes the startup log, builds the worker pool, manages
  core's state before any plugin sets up, registers the plugins, then starts the transport server and builds the main
  window. A failure before the window ends the process; a Windows release build, which has no console, says why in a
  message box.
- `plugins`: the plugins the application registers, in setup order: `render` first, since the domain plugins take the
  units it manages, then core's own plugins and every domain.
- `ipc`: the command surface composed from every plugin crate's `ipc/domains.rs`. Its `registry` joins their transport
  routes at run time and holds the build-time declarations `build.rs` includes; `bindings` writes the frontend's
  TypeScript mirrors and exists only in a test build with the `typescript-bindings` feature.
- `window`: the main window: how it is built, the geometry it reopens at, the reveal that shows it when the page does
  not, the build kind it shows, and the unpacked browser extensions a debug build loads.
- `preferences`: what the backend keeps between runs in a store file in the application's data, as opposed to what the
  page keeps in its own storage. It holds the main window's geometry.

Beside `src/`: `build.rs` declares the inline plugins and their permissions from every plugin crate's `domains.rs`,
records the build information and sets the executable's icon by build kind; a DLSS build also links what NGX's library
needs and merges `tauri.dlss.conf.json`, which adds the DLSS runtime to the renderer's bundle. `tauri.conf.json`
configures the application and `tauri.windows.conf.json` adds to it on Windows. `capabilities/` holds the window's
permissions, `icons/` the icons of each build kind, and `gen/` the schemas Tauri generates. `extensions/README.md`
describes loading browser extensions into a debug build.

## Features

- `custom-protocol`: Tauri's feature for production builds, or when no dev server is set.
- `tracy`: zones, GPU zones and frame marks for a Tracy capture, built with the `tracy` profile.
- `dlss`: NVIDIA DLSS upscaling on Vulkan and D3D12, built against its SDK as `DEVELOPMENT.md` sets it up.
- `typescript-bindings`: emits the TypeScript mirrors and command bindings of every plugin crate. Nothing in a normal
  build turns it on.

## Checks

Run `cargo test -p xrf-app` from the repository root. Its tests set up every plugin over the state managed before it,
check the declared inline plugins against the registered ones, and cover the window's geometry.
