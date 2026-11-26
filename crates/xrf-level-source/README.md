# xrf-level-source

A compiled level as a host opens it and as [xrf-world](../xrf-world/README.md) plays it: a minimal root and the parts
read apart from it, each kept with what it was read from, and the `WorldLevelSource` the world streams and
[xrf-renderer](../xrf-renderer/README.md) draws. It knows no application shell: the desktop app opens levels as its
documents through it, and any other native host shows a level from the same pieces. The level files themselves are
read by `xrf-level`, the spawn by `xrf-spawn` and the weather by `xrf-environment`.

## Opening and showing a level

1. `OpenedLevel::read(search, source, roots, dialect, engine)` reads the level a `LevelSource` names, through a search
   of the [xrf-vfs](../xrf-vfs/README.md) host, and answers a `SessionDerived<SelectedLevel>`: the root, what it was
   read from, and the parts read on the way.
2. A host keeps that as an [xrf-session](../xrf-session/README.md) `SessionSnapshot<SelectedLevel>`.
3. `LevelRenderSource::new(snapshot, host, audio)` is the source the world shows, every read it makes going through
   the shared `XrayVfsHost`; `follow(next)` draws a later snapshot of the same session with the textures resolved so
   far.

`OpenedLevel::derive(snapshot, host)` reads an open level again over what it was opened with, for a host that follows
its files.

## Layout

- `level`: a level as a document: where it is read from (`LevelSource`), its `level` bundle, how an open reads it and
  reads it again (`OpenedLevel`), and what a host is shown of one.
- `parts`: what an open level reads beside its bundle, each a part of its session with its value beside it, one part
  type to a file (`Level<Part>Part`): from the level's own files (`level`), the game's spawn and what it names
  (`spawn`), what the renderer draws beside the sectors (`drawn`), the game's environment (`environment`) and its
  console defaults (`console`). A change to what a part read moves that part and not the level.
- `objects`: a level's objects (`LevelObject`) as an outliner, picks and helpers address them (`LevelObjectId`), read
  from its share of the spawn, its companion files and its static lights.
- `spawn`, `weather`, `console`: a level's spawn objects and why a new game releases one, its weather cycles and
  effects (with `weather/render` holding the rain, wet surfaces and thunderbolts only the renderer reads), and the
  console values it is lit and exposed with.
- `sound`: a shown level's sound: its static and zone emitters and music (`LevelWorldSounds`), the sound files it names
  (`LevelSoundFilesPart`), the collision geometry with each material's acoustics (`LevelSoundGeometry`), the sink that
  carries the world's sound to the audio thread, and the tee a host arms to copy a burst's frames (`BurstSoundTee`).
- `render`: a level drawn by the native renderer: its render source, which of its parts feed the scene
  (`LevelRenderParts`), its audio (`LevelAudio`) and its particle collider.
- `visuals`: spawned objects' visuals: `SpawnVisualReader` reads each beside the level, then in the shared meshes,
  packed, posed in the idle rest pose and dressed, into the `SpawnVisuals` a session hands it.
- `helpers`: how helpers look: a link's arrowed lines and a kind's colour.
- `report`: what reading and showing a level writes to the log.

## Errors

Every reader answers `XrfResult`, the error's kind saying what went wrong: a read that failed (`Read`), a file or name
that is not there (`NotFound`), a name or input refused (`Invalid`). A host showing one to a person turns it to text
where it crosses, as the app's levels plugin does into its `TauriResult`; the renderer's and the world's traits take it
as it is.

## Features

- `typescript-bindings` derives the Specta types and registers the constants of what crosses to a page.
- `fixtures` exposes the test builders a host's tests share with this crate's (`fixtures`: levels, searches, catalogs
  and spawned objects).

## Checks

Run `cargo test --locked -p xrf-level-source` from the repository root.
