# xrf-app-render

The native viewports `xrf-renderer` draws into the XRF desktop application's windows, under their webviews: the
renderer, the world it draws, what each viewport shows and the windows hosting them, captures, and texture reloads. It
is infrastructure for the domain plugins and knows no domain: a domain such as `xrf-app-levels`, `xrf-app-visuals` or
`xrf-app-textures` hands its own render source to the units here.

## Layout

- `renderer/`: `Renderer`, the renderer as the application calls it, each call a command on the render thread's queue
  and each question of a viewport answered through `RenderAnswer` on a blocking thread of its own; the request a pick
  takes (`RenderPickRequest`); and `bundle/`, the renderer's bundle as the application ships it (or the renderer's
  checkout in a development build) and the folder of the application's cache the driver's compiled pipelines are kept
  in between runs.
- `world/`: `WorldState`, the world the renderer draws, which the camera, weather and show commands steer.
- `viewport/`: the viewports in each window (`RenderViewports`), what each was last asked to show (`ViewportShows`),
  the window each draws into, the events a viewport sends its page, and `window_host/`, the native windows viewports are
  drawn into, one host per platform the renderer presents on (Win32 and macOS).
- `texture/`: drawn textures kept current with their files (`TextureReloads`): the shown roots watched for texture
  files written by anything, and where a render source's texture references resolve (`TextureResolver`).
- `capture/`: frames read back and written to PNG files, and how a many-shot capture names itself and ends; the level
  plugin's map and cubemap captures use them.
- `commands/`: one adapter a command, by what it addresses: `viewport/` (attach, detach, layout, capture), `view/` (view
  options, overlays, selection), `renderer/` (configuration, the clock, backends, libraries, profiling marks),
  `queries/` (picks and descriptions of the frame, loads, textures, problems and surfaces), `camera/` (the camera,
  input and model posing) and `world/` (weather and effect playback, world toggles).
- `plugin` and `ipc`: `RenderPlugin` and its command domain.
- `tests/`: the crate's tests.

## In the application

`RenderPlugin` is set up before every other plugin, since the domain plugins take the units it manages: `Renderer`
(its loaders running on the application's pool), `WorldState`, `ViewportShows`, `RenderViewports` and
`TextureReloads`. Its hooks detach a window's viewports when a page starts loading over them or the window closes,
hold a closing window open until the renderer lets go of it, and stop the renderer as the application exits. The
`render` domain runs no jobs.

## Checks

Run `cargo test -p xrf-app-render` from the repository root.
