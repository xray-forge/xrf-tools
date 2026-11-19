# xrf-app-spawn

`all.spawn` files in the XRF desktop application: a spawn opened as a session, read chunk by chunk, saved, and packed or
unpacked as jobs, and a root spawn built from a game's levels as a job. The spawn format is `xrf-spawn` and the build
`xrf-spawn-builder`; this crate holds an opened spawn, answers the editors and runs the build.

## Layout

One type to a file, grouped by concern:

- `file/`: the opened spawn.
  - `SpawnFiles`: the spawns open in the editors, each held by the tool that opened it.
  - `SpawnSession` and `SpawnSessionDescriptor`: one opened spawn as read, and what describes it without its large
    chunks, which an open and a restore answer.
  - `SpawnChunk` and `SpawnChunkKind`: one large chunk of an opened spawn, and which one to read.
- `conversion/`: `SpawnConversion` and `SpawnConversionResult`, packing a directory of chunks into one file or
  unpacking a file into one, run as a job on one lane shared by both, and what reached disk. A conversion gives up only
  before it starts writing.
- `build/`: `SpawnBuild` and `SpawnBuildResult`, a root spawn built from a game's levels as `xrf-cli spawn build`
  builds it, and how it ended. It runs as a job one at a time, cancellable between stages but not once it writes.
- `request/`: `SpawnConvertRequest`, the source and output paths of a conversion.
- `commands/`: opening, restoring and closing a spawn; reading one chunk; saving a spawn as a packed file or as a
  directory of unpacked chunks; packing and unpacking; and listing the levels a spawn can be built from and building
  one.
- `plugin` and `ipc`: `SpawnPlugin` and its command domain.
- `tests`: the crate's tests and a synthetic spawn they share.

## In the application

`SpawnPlugin` manages `SpawnFiles`. The `spawn` domain runs the jobs `spawn.build`, `spawn.pack` and `spawn.unpack`.

## Checks

Run `cargo test -p xrf-app-spawn` from the repository root.
