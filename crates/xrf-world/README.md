# xrf-world

The world a renderer draws: what moves and plays in a level whatever draws it — the cameras, the weather, the level's
streaming into its scene and its skinned objects' poses — run inline on the render thread before each frame and handed
to [xrf-renderer](../xrf-renderer/README.md) as one frame's input. The renderer draws; this crate decides what is there
to draw. The desktop app owns one `World`, shares it with the renderer as its `RenderWorld`, and steers it from its
viewport commands.

## The world and its viewports

`World::new(workers)` makes a world that reads its levels and weather on the renderer's `RenderWorkers` pool. Everything
it holds is keyed by `RenderViewportId`:

- `attach(viewport, sink)` starts a viewport's world and is the only way one starts: a command or a frame naming a
  viewport not attached, or detached since, does nothing. The `WorldEventSink` receives the viewport's
  `WorldViewportEvent`s: `Camera` while the camera moves and once after it stops, and `Weather` as the weather changes,
  a few times a second at most. `detach` ends it.
- `set_camera` describes the viewport's camera, a `WorldCamera`: `Fly`, turned by a drag and moved by the keys as a
  level is walked, or `Orbit` around a target, as a model or texture preview. Described again from the same start, a
  camera keeps where it has been moved. `command_camera` and `send_input` move it.
- `set_toggles` sets what plays from the next frame (`WorldToggles`): rain, bolts, wind, campfires, ambient effects, and
  which groups of spawned objects stream in. Defaults play everything but the objects a new game releases.

## Levels and scenes

`show_level(viewport, source)` shows a level, a `WorldLevelSource`, in a viewport and plays its weather there, or shows
nothing with `None`. The app implements the trait: it extends the renderer's `RenderLevelSource` with every read the
world makes (sectors packed, lights, spawn and its models, grass, particles, motions, and the level's weather, a
`WorldLevelWeather` whose `render` part the renderer draws with), each called from loader threads, several side by
side. It answers the `RenderSceneId` that load reports name:

- Viewports whose sources answer the same `get_scene_key` share one scene, streamed, posed and played once for all of
  them. The first viewport showing it drives it: the scene streams around that viewport's camera and stands its skinned
  objects as that viewport asks (`pose_model`). When the driver leaves, the next viewer drives; when the last leaves,
  the scene goes.
- `reload_scene(key, source, reload)` moves a shared scene onto a next source of the same level, keeping its id and
  viewers: only what `WorldLevelReload` names is read again and replaces what was drawn once read.
- `reopen_scene(key, source)` streams the level afresh as a new scene, which every viewer of the old one then shows.
- `measure_surfaces` and `describe_problems` report on the level a viewport shows: how much each shader table entry
  draws, and what could not be drawn as asked.

Weather plays per viewport: `play_weather`, `set_weather_control`, `seek_weather`, `play_weather_effect`, and
`play_ambient_effect` near the driving camera (never indoors).

## Each frame

As the renderer's `RenderWorld`, the world answers the render thread before each frame: `advance_view` moves each
viewport's camera and weather and answers what its view draws, then `advance_scenes` moves every scene to the frame's
time by its driving viewport and answers its updates (streamed sectors and models, poses, effects, progress), taking in
what the renderer reported of the last frame (sector and model failures, finished effects). A frame for a viewport not
attached rests. The world allocates the handles of the items it posts with `xrf-renderer-core`'s `ProxyAllocator`, so
the renderer inserts each at the handle the world already holds. When the renderer lets go of its GPU,
`restart_scenes` streams every scene in again from the start.

The contract types (`World*` under `src/contract/`) cross IPC to the page; the `typescript-bindings` feature derives
their Specta types for the generated mirrors. What the host provides (`src/host/`: the level source, its weather and
ambients, the event sink) stays in the process.

## Checks

Run `cargo test --locked -p xrf-world`. Sources: [`World`](src/world.rs), its
[level source](src/host/world_level_source.rs), the [level](src/level/world_level.rs) and its
[streaming](src/level/streaming/), the [weather](src/weather/) and the [cameras](src/camera/).
