# xrf-renderer-host

What a native host hands [xrf-renderer](../xrf-renderer/README.md) beside its levels: the windows viewports present
into, the bundle of files the renderer ships with beside its code, where its textures are found, and its pipeline
cache on disk. It knows no application shell, so the desktop app and any other native host build their renderer from
the same pieces.

## Windows

`unsafe { WindowHostBuilder::new(handle) }.build()` makes the `RenderWindowHost` for a window's native handle:

| Platform | Handle     | Host                                                                                   |
| -------- | ---------- | -------------------------------------------------------------------------------------- |
| Windows  | The `HWND` | `Win32WindowHost`: the swapchain on the window itself, its client area read per frame  |
| macOS    | `NSWindow` | `MacosWindowHost`: the `CAMetalLayer` of an `XrfRendererView` placed under the content |
| Other    | —          | None: `build` fails                                                                    |

- The macOS host is made on the main thread only, where AppKit can be touched; `build` fails anywhere else.
- `WindowHostBuilder::new` is unsafe, because `RenderWindowHost` is an unsafe contract the hosts cannot keep alone:
  the caller promises the handle names a live window, kept alive until the render thread has handled the detach of
  every viewport drawn into it.
- Windows is the only platform verified; the macOS host compiles for it and is not run.

A host that owns its window takes the safe route: `HostedWindow::new(window)` holds any window with a
`raw-window-handle` handle, and `get_host` answers the host to attach its viewports with. It keeps the window alive
while any host of it is held, so the window outlives every frame drawn into it without the caller's promise. Detach the
window's viewports, wait for `RenderQueue::wait_handled` to answer, then drop the `HostedWindow` on the thread that made
the window: it drops the window there. Dropped while a host is still held, it leaks the window rather than close it
under the renderer, and logs an error. An application whose window type is a cheap clone, as SDL's is, hands it a
clone and keeps its own to change the window through. It hosts Win32 windows only; elsewhere `new` fails.

## The bundle

`PackagedBundle::locate(resource_dir)` reads the renderer's bundle from `resources/renderer` under a packaged
application's resource folder, or from the renderer's checkout (`RENDER_BUNDLE_SOURCE`) where nothing was packaged, as
a development build runs. It logs where it reads from.

## Textures

`TextureResolver::new(roots, scope, known)` is where a render source's texture references are found and read: its
roots and texture scope, and the file each reference resolved to, known ahead (a `ResolvedTexture` each, as a level
resolves its own at open) or once first asked for. `read`, `read_sequence` and `get_stamp` take the
[xrf-vfs](../xrf-vfs/README.md) host they search through; `forget` lets the next read find a file afresh.

## The pipeline cache

`DiskCacheStore::new(store)` keeps the driver's compiled pipelines between runs as the renderer's `RenderCacheStore`,
in one client's folder of a host's caches ([xrf-utils](../xrf-utils/README.md)'s `AppCacheStore`).

## Checks

Run `cargo test --locked -p xrf-renderer-host` from the repository root.
