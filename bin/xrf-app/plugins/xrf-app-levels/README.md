# xrf-app-levels

Compiled levels in the XRF desktop application, opened as documents: a minimal root and the parts read apart from it,
each with its own query and followed as its files change; shown in a viewport through `xrf-app-render`'s units; and
captured or exported through the level viewer. The level files themselves are read by `xrf-level`, the spawn by
`xrf-spawn` and the weather by `xrf-environment`. Level Editor scenes, the source a level is built from, open as
documents beside them, read by `xrf-editable-level`, and are shown in a viewport too, each scene object a model compiled
from its `.object` by `xrf-object-visual`.

## Layout

- `level`: a level as a document: where it is read from (`LevelSource`), its `level` bundle, how an open reads it and a
  commit reads it again (`OpenedLevel`), the levels open (`SelectedLevels`), and what the page is shown of one.
- `parts`: what an open level reads beside its bundle, each a part of its session with its value beside it, one part
  type to a file (`Level<Part>Part`): from the level's own files (`level`), the game's spawn and what it names
  (`spawn`), what the renderer draws beside the sectors (`drawn`), the game's environment (`environment`) and its
  console defaults (`console`). A commit that changes what a part read moves that part and not the level.
- `spawn`, `weather`, `console`: the page's answers about a level's spawn objects and why a new game releases one, its
  weather cycles and effects (with `weather/render` holding the rain, wet surfaces and thunderbolts only the renderer
  reads), and the console values it is lit and exposed with.
- `render`: a level drawn by the native renderer: its render source, the levels shown (`ShownLevels`), how a shown
  level follows a commit (`LevelSceneFollower`), and its ambients, lens flares and particle collider.
- `viewer`: the level viewer's own files in the application's data: its views, screenshots, and the level map and
  cubemap captures and their exports, with the map's annotations of logic volumes, patrols and graph points.
- `scene`: a Level Editor scene as a document: where it is read from (`SceneSource`, a `.level` whose directory roots
  its search), how an open reads its `.level` and each tool's part files and a commit reads it again (`OpenedScene`),
  the scenes open (`SelectedScenes`), the game data each is drawn with (`SceneGameData`) and what the page is shown of
  one (`document`, `description`); its object tools' typed objects, each group's members under their own tool
  (`objects`, `SceneObjectsPart`), and its binary tools' counts (`binary`, `SceneBinaryToolsPart`), each a part with
  its own query; what the editor would skip or refuse of a part, kept as a problem on its tool (`problem`); the scene
  objects it draws, their extent, each one's sphere and each it does not draw, its `.object` missing or unreadable
  (`draw`, `SceneDrawPart`); the spawn points drawn as their entities' models (`spawn`): each whose packet names a
  visual (`LevelSpawnDataFields::read_visual`, as the SDK's editor draws it) that the scene's game data holds, looked
  up and not read (`SceneSpawnModelsPart`), and the visuals they stand as, read through the shared `SpawnVisualReader`
  into the scene's own `SceneSpawnVisualsPart`; the helpers of what has no geometry (`helpers`, `SceneHelpersPart`,
  read from the parts the open kept and which spawn points stand as models): one a helped tool's object in its tool's
  row of `SCENE_HELPER_TOOLS`, switched by the tool's place in `SceneToolKind::ALL`, a spawn point drawn as its model
  keeping its attached volume and losing its icon, with their extent and per-tool counts for the page; and the scene as
  the renderer draws it (`render`): `SceneRenderSource`, no sectors, every scene object whose `.object` is found a model
  in the props group and every spawn point drawn as its model in the items group, each lit by the open sky, a model
  that does not compile or read left out as a load failure named by its reference or its visual's file, which the
  viewport's problems answer as for any source, the helpers part as its helpers; one id every drawn thing of an object
  carries, model, lighting, hiding, selection, helper and pick alike (`SceneRenderId`, the tool's place above
  `SCENE_RENDER_INDEX_BITS` of the object's index); the space it all stands in (`SceneRenderSpace`, engine space
  mirrored through `z`), where one object stands for a camera to frame it, its drawn box, its spawn model's placed
  bounds or its helper's extent (`SceneObjectSphere`), and the scenes shown (`ShownScenes`).
- `visuals`: what spawned objects' visuals are read as wherever the plugin draws them, a compiled level's spawn and a
  scene's spawn points alike: `SpawnVisualReader` (beside the level then in the shared meshes, packed, posed in the
  idle rest pose and dressed) filling the `SpawnVisuals` the session hands it, with the search its game data is read
  through; `LevelSpawnVisualsPart` and `SceneSpawnVisualsPart` each keep a session's own.
- `helpers`: what the helpers look like wherever the plugin draws them, the compiled level's objects and a scene's
  alike: a link's arrowed lines and a kind's colour.
- `commands/`: the level commands: listing, opening, restoring, closing and showing a level, and describing its map,
  start, surfaces, textures, spawn objects, weather and console defaults; and in `commands/scene/` the scene's:
  opening, restoring, closing and showing a scene, describing its objects, binary tools, drawing and helpers, and
  locating one object. The
  viewer's commands are in `viewer/commands/`.
- `level_report`: what the plugin writes to the log.
- `plugin` and `ipc`: `LevelsPlugin` and its command domain.
- `tests/`: the crate's tests and their fixtures.

## In the application

`LevelsPlugin` manages `SelectedLevels`, `SelectedScenes`, `ShownLevels` and `ShownScenes`, and needs the render plugin
set up first, since a shown level's scene is drawn in its `WorldState`. An open level records what it and its parts
read: a commit carries the parts it did not touch and reads the level again on the pool where it moved one of the
level's own files. An open scene records its `.level`, its part directory's listing and every part file it read, so a
commit to any of them, or a part file come or gone, reads it again on the pool; its objects and drawing parts record
where they looked for the scene objects' library objects and what they read of them, so an `.object` come, gone or
changed moves those parts alone. A shown scene does not follow a commit yet: it stays drawn from the snapshot shown
until the page shows the session again. The `levels` domain serves the bulk route `read_screenshot` and runs the jobs
`levels.capture-map` and `levels.capture-cube`.

## Checks

Run `cargo test -p xrf-app-levels` from the repository root.
