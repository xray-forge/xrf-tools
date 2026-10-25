# xrf-app-spawn

`all.spawn` files in the XRF desktop application: a spawn opened as a session, read chunk by chunk, saved, and packed or
unpacked as jobs. The spawn format is `xrf-spawn`; this crate holds an opened spawn and answers the editors.

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
- `request/`: `SpawnConvertRequest`, the source and output paths of a conversion.
- `commands/`: opening, restoring and closing a spawn; reading one chunk; saving a spawn as a packed file or as a
  directory of unpacked chunks; and packing and unpacking.
- `plugin` and `ipc`: `SpawnPlugin` and its command domain.
- `tests`: the crate's tests and a synthetic spawn they share.

## In the application

`SpawnPlugin` manages `SpawnFiles`. The `spawn` domain runs the jobs `spawn.pack` and `spawn.unpack`.

## Checks

Run `cargo test -p xrf-app-spawn` from the repository root.
