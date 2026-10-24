# xrf-app-archives

Every archive surface of the XRF desktop application: volume sets and game folders browsed, read, described and
extracted, and trees packed, patched and unpacked as jobs. It is one plugin rather than five because the surfaces share
a wire namespace and a publication lease, and five directories rather than one flat `commands/` so that the commands
that belong together sit together.

## Layout

- `browse`: what the explorer has open (`ArchiveSubject`, a volume set or a game folder, and the world of entries it
  lists), and reading or extracting out of it.
- `describe`: saying in words what an entry the viewer cannot draw contains, one describer per format: `anm`,
  `animation` (envelopes), `chunks` (a container nothing else reads), `detail`, `efd`, `gamemtl`, `level` (a module per
  compiled level file), `light_anim`, `omf`, `particles`, `ppe`, `shader_compiler`, `shaders`, `sound`, `spawn` and
  `thm`. A describer gets what it needs through `ArchiveDescribeSource`, so it is written against its format rather
  than against the subjects the explorer browses.
- `preview`: decoding one entry of the open subject for display: a sound's or a picture's facts, its stored bytes for
  the page to play or draw, and a texture as a picture.
- `pack`: writing a tree into archive volumes, and the configuration a run is authored in.
- `patch`: comparing two trees and publishing the difference as a patch volume.
- `unpack`: writing a volume set back out to a directory whole.
- `lease`: the one publication lane packing and patching share.
- `plugin` and `ipc`: `ArchivesPlugin` and its command domain.

## In the application

`ArchivesPlugin` manages `ArchiveSubjects`, the subjects open in each owner scope; a subject is listed again on the
pool once a commit moves what it read. The `archives` domain serves the bulk routes `read_entry` and `read_texture` for
an entry's bytes and runs the jobs `archives.compare`, `archives.extract`, `archives.pack`, `archives.patch` and
`archives.unpack`.

## Checks

Run `cargo test -p xrf-app-archives` from the repository root.
