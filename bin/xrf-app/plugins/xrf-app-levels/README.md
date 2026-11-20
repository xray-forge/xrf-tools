# xrf-app-levels

Compiled levels in the XRF desktop application, opened as documents: a minimal root and the parts read apart from it,
each with its own query and followed as its files change; shown in a viewport through `xrf-app-render`'s units; and
captured or exported through the level viewer. The level files themselves are read by `xrf-level`, the spawn by
`xrf-spawn` and the weather by `xrf-environment`. Level Editor scenes, the source a level is built from, open as
documents beside them through `xrf-editable-level` and show in a viewport too, each scene object a model compiled from
its `.object` by `xrf-object-visual`. A scene builds into a level, and a level decompiles into a scene, as jobs.

## Layout

- `level`: a level as a document: where it is read from (`LevelSource`), its `level` bundle, how an open reads it and a
  commit reads it again (`OpenedLevel`), the levels open (`SelectedLevels`), and what the page is shown of one.
- `parts`: what an open level reads beside its bundle, each a part of its session with its value beside it, one part
  type to a file (`Level<Part>Part`): from the level's own files (`level`), the game's spawn and what it names
  (`spawn`), what the renderer draws beside the sectors (`drawn`), the game's environment (`environment`) and its
  console defaults (`console`). A commit that changes what a part read moves that part and not the level.
- `objects`: a level's objects (`LevelObject`) as the Outliner, picks and helpers address them (`LevelObjectId`), read
  from its share of the spawn, its companion files and its static lights.
- `spawn`, `weather`, `console`: the page's answers about a level's spawn objects and why a new game releases one, its
  weather cycles and effects (with `weather/render` holding the rain, wet surfaces and thunderbolts only the renderer
  reads), and the console values it is lit and exposed with.
- `sound`: a shown level's sound: its static and zone emitters and music (`LevelWorldSounds`), the sound files it names
  (`LevelSoundFilesPart`), the collision geometry with each material's acoustics (`LevelSoundGeometry`), the sink that
  carries the world's sound to the audio thread, and the report joining both.
- `render`: a level drawn by the native renderer: its render source, the levels shown (`ShownLevels`), how a shown
  level follows a commit (`LevelSceneFollower`), and its ambients, lens flares and particle collider.
- `viewer`: the level viewer's own files in the application's data: its views, screenshots, and the level map and
  cubemap captures and their exports, with the map's annotations of logic volumes, patrols and graph points.
- `scene`: a Level Editor scene as a document, each piece a part with its own query:
  - `document`, `description`: where it is read from (`SceneSource`, a `.level` whose directory roots its search), how
    an open reads the `.level` and each tool's part files (`OpenedScene`), the scenes open (`SelectedScenes`), the game
    data each is drawn with (`SceneGameData`), and what the page is shown of one.
  - `objects`, `binary`, `problem`: the object tools' typed objects, each group's members under their own tool
    (`SceneObjectsPart`); the binary tools' counts (`SceneBinaryToolsPart`); and what the editor would skip or refuse
    of a part, kept as a problem on its tool.
  - `draw`: the scene objects drawn, their extent and spheres, and each not drawn because its `.object` is missing or
    unreadable (`SceneDrawPart`).
  - `spawn`: the spawn points drawn as their entities' models, as the SDK's editor draws them: each whose packet names
    a visual (`LevelSpawnDataFields::read_visual`) the scene's game data holds (`SceneSpawnModelsPart`), and those
    visuals, read through the shared `SpawnVisualReader` (`SceneSpawnVisualsPart`).
  - `helpers`: what has no geometry, drawn as helpers (`SceneHelpersPart`), one per object of a tool listed in
    `SCENE_HELPER_TOOLS`; a spawn point drawn as its model keeps its attached volume and loses its icon.
  - `render`: the scene as the renderer draws it (`SceneRenderSource`): no sectors, scene objects in the props group
    and spawn models in the items group, lit by the open sky. A model that does not compile or read is left out as a
    load failure, reported in the viewport's problems. `SceneRenderId` is the one id every drawn thing of an object
    carries (model, lighting, hiding, selection, helper and pick): the tool's place in `SceneToolKind::ALL` above
    `SCENE_RENDER_INDEX_BITS` of the object's index. `SceneRenderSpace` is engine space mirrored through `z`;
    `SceneObjectSphere` frames one object for a camera; `ShownScenes` holds the scenes shown.
- `visuals`: spawned objects' visuals for a compiled level's spawn and a scene's spawn points alike: `SpawnVisualReader`
  reads each beside the level, then in the shared meshes, packed, posed in the idle rest pose and dressed, into the
  `SpawnVisuals` a session hands it. `LevelSpawnVisualsPart` and `SceneSpawnVisualsPart` keep each session's own.
- `helpers`: how helpers look for a compiled level and a scene alike: a link's arrowed lines and a kind's colour.
- `rawdata`: building a scene into a level, with an optional compile after it (`LevelBuildRun`), and decompiling a
  level into a scene, run as jobs, and how each ended.
- `commands/`: listing, opening, restoring, closing and showing a level; describing its map, start, surfaces, textures,
  spawn objects, weather and console defaults; reading any weather cycle; building and decompiling; and in `sound/`
  the shown level's sound and its music. `commands/scene/` opens, restores, closes and shows a scene, describes its
  objects, binary tools, drawing and helpers, and locates one object. The viewer's commands are in `viewer/commands/`.
- `report`: what the plugin writes to the log.
- `plugin` and `ipc`: `LevelsPlugin` and its command domain.
- `tests/`: the crate's tests and their fixtures.

## In the application

`LevelsPlugin` manages `SelectedLevels`, `SelectedScenes`, `ShownLevels` and `ShownScenes`, and needs the render plugin
set up first, since a shown level's scene is drawn in its `WorldState`.

- An open level records what it and its parts read: a commit carries the parts it did not touch and reads the level
  again on the pool where it moved one of the level's own files.
- An open scene records its `.level`, its part directory's listing and every part file it read; a commit to any of
  them, or a part file added or removed, reads it again on the pool. Its objects and drawing parts record where they
  looked for library objects and what they read, so an `.object` added, removed or changed moves those parts alone.
- A shown scene stays drawn from the snapshot shown until the page shows the session again; it does not follow a
  commit.

The `levels` domain serves the bulk route `read_screenshot` and runs the jobs `levels.build`, `levels.decompile`,
`levels.capture-map` and `levels.capture-cube`.

## Checks

Run `cargo test -p xrf-app-levels` from the repository root.
