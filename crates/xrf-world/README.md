# xrf-world

Decides what is in a level each frame, for [xrf-renderer](../xrf-renderer/README.md) to draw: the cameras, the
weather, the level streaming into its scene, and its skinned objects' poses.

It runs inline on the render thread before each frame and hands the renderer one frame's input. The renderer draws;
this crate decides what there is to draw.

The desktop app owns one `World`, shares it with the renderer as its `RenderWorld`, and steers it from its viewport
commands.

## The world and its viewports

`World::new(workers)` makes a world that reads levels and weather on the renderer's `RenderWorkers` pool. Everything it
holds is keyed by `RenderViewportId`.

- **`attach(viewport, sink)`** starts a viewport's world; it is the only way one starts. A command or frame naming a
  viewport not attached (or since detached) does nothing. `detach` ends it.
- **The event sink** (`WorldEventSink`) receives the viewport's `WorldViewportEvent`s: `Camera` while the camera moves
  and once after it stops, and `Weather` as the weather changes, a few times a second at most.
- **`set_camera`** describes the viewport's camera, a `WorldCamera`:
  - `Fly`, turned by dragging and moved by keys, for walking a level;
  - `Orbit` around a target, for a model or texture preview.

  Described again from the same start, a camera keeps where it has been moved. `command_camera` and `send_input` move
  it.

- **`set_toggles`** sets what plays from the next frame (`WorldToggles`): rain, lightning, wind, campfires, ambient
  effects, and which groups of spawned objects stream in. By default everything plays except the objects a new game
  releases.

## Levels and scenes

`show_level(viewport, source)` shows a level in a viewport and plays its weather there; `None` shows nothing. It returns
the `RenderSceneId` that load reports name.

The source is a `WorldLevelSource`, which the app implements. It extends the renderer's `RenderLevelSource` with every
read the world makes: packed sectors, lights, spawns and their models, grass, particles, motions, and the level's
weather (a `WorldLevelWeather` whose `render` part the renderer draws with). Each read is called from loader threads,
several at once.

### Shared scenes

Viewports whose sources return the same `get_scene_key` share one scene, streamed, posed and played once for all of
them.

- The first viewport showing it drives it: the scene streams around that viewport's camera and poses its skinned
  objects as that viewport asks (`pose_model`).
- When the driver leaves, the next viewer drives. When the last leaves, the scene is dropped.
- `reload_scene(key, source, reload)` moves a shared scene to a new source for the same level, keeping its id and
  viewers. Only the parts `reload` names (the renderer's `LevelParts`) are read again, each replacing what was drawn once
  read.
- `reopen_scene(key, source)` streams the level afresh as a new scene, which every viewer of the old one then shows.

### Reports

- `measure_surfaces` reports how much each shader table entry draws in the level a viewport shows.
- `describe_problems` reports what could not be drawn as asked.

### Weather

A scene reads its level's weather once; every viewer plays from that read. Weather plays per viewport:

- `play_weather`, `set_weather_control`, `seek_weather`;
- `play_weather_effect`;
- `play_ambient_effect`, near the driving camera and never indoors.

## Each frame

As the renderer's `RenderWorld`, the world answers the render thread before each frame:

1. **`advance_view`** moves each viewport's camera and weather and returns what its view draws. A frame for a viewport
   not attached does nothing.
2. **`advance_scenes`** moves every scene to the frame's time by its driving viewport and returns its updates
   (streamed sectors and models, poses, effects, progress). It takes in what the renderer reported about the last frame:
   sector and model failures, finished effects.

- The world allocates the handles of the items it posts with `xrf-renderer-core`'s `ProxyAllocator`, so the renderer
  inserts each at the handle the world already holds.
- When the renderer releases its GPU, `restart_scenes` streams every scene in again from the start.

## Contract and host

- The contract types (`World*` under `src/contract/`) cross IPC to the page. The `typescript-bindings` feature derives
  their Specta types for the generated mirrors.
- What the host provides (`src/host/`: the level source, its weather and ambients, the event sink) stays in-process.

## Checks

Run `cargo test --locked -p xrf-world`.

Sources: [`World`](src/world.rs), its [level source](src/host/world_level_source.rs), the
[level](src/level/world_level.rs) and its [streaming](src/level/streaming/), the [weather](src/weather/) and the
[cameras](src/camera/).
