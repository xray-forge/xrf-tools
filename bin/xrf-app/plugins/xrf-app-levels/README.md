# xrf-app-levels

Compiled levels in the XRF desktop application, opened as documents: a minimal root and the parts read apart from it,
each with its own query and followed as its files change; shown in a viewport through `xrf-app-render`'s units; and
captured or exported through the level viewer. The level files themselves are read by `xrf-level`, the spawn by
`xrf-spawn` and the weather by `xrf-environment`. Level Editor scenes, the source a level is built from, open as
documents beside them, read by `xrf-editable-level`.

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
  the scenes open (`SelectedScenes`) and what the page is shown of one (`document`, `description`); its object tools'
  typed objects, each group's members under their own tool (`objects`, `SceneObjectsPart`), and its binary tools'
  counts (`binary`, `SceneBinaryToolsPart`), each a part with its own query; and what the editor would skip or refuse
  of a part, kept as a problem on its tool (`problem`).
- `commands/`: the level commands: listing, opening, restoring, closing and showing a level, and describing its map,
  start, surfaces, textures, spawn objects, weather and console defaults; and in `commands/scene/` the scene's:
  opening, restoring and closing a scene, and describing its objects and binary tools. The viewer's commands are in
  `viewer/commands/`.
- `level_report`: what the plugin writes to the log.
- `plugin` and `ipc`: `LevelsPlugin` and its command domain.
- `tests/`: the crate's tests and their fixtures.

## In the application

`LevelsPlugin` manages `SelectedLevels`, `SelectedScenes` and `ShownLevels`, and needs the render plugin set up first,
since a shown level's scene is drawn in its `WorldState`. An open level records what it and its parts read: a commit
carries the parts it did not touch and reads the level again on the pool where it moved one of the level's own files.
An open scene records its `.level`, its part directory's listing and every part file it read, so a commit to any of
them, or a part file come or gone, reads it again on the pool; its objects part records where it looked for the scene
objects' library objects, so an `.object` come or gone moves that part alone. The `levels`
domain serves the bulk route `read_screenshot` and runs the jobs `levels.capture-map` and `levels.capture-cube`.

## Checks

Run `cargo test -p xrf-app-levels` from the repository root.
