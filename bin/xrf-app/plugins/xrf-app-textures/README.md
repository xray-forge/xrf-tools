# xrf-app-textures

Textures as the engine names them, in the XRF desktop application: the catalog of a root set, what one texture's
descriptor makes of it, the bytes a webview needs to show it, and the writing the editor does to all three. Each concern
owns its commands, its requests and its state. The texture and descriptor formats are `xrf-texture` and `xrf-thm`.

## Layout

- `describe`: reading one texture: how it is named, the two files it is (the `.dds` and the `.thm` describing it),
  everything the panels say about it, and the names the SDK gives the numbers its descriptor stores (`vocabulary`).
- `browse`: every texture a root set holds, listed by reference or by path, with the badges each descriptor earns
  (`summary`); `TextureBrowses` holds the open browses.
- `edit`: the descriptor fields a person edits, where an edit lands with the stamp of what is there, saving a texture's
  two files or refusing to, and rebuilding a texture or its bump pair from images.
- `encoding`: the candidate formats a texture could be written in, weighed against it, and the encodes a comparison
  keeps (`TextureComparisons`) until one is saved over the texture they were made of.
- `surface`: a texture laid on a plane, a sphere or a cube, drawn by the native renderer through `xrf-app-render`.
- `lease`: the one encode run admitted at a time, since building, bump generation and comparison each saturate the
  shared pool.
- `plugin` and `ipc`: `TexturesPlugin` and its command domain.
- `tests/`: the crate's tests and their fixtures.

## In the application

`TexturesPlugin` manages `TextureBrowses` and `TextureComparisons`, each closed with the owner scope that holds it. The
`textures` domain serves the bulk routes `read_texels` (a texture's top level as plain texels) and `read_candidate` (one
weighed candidate as a picture), and runs the jobs `textures.build`, `textures.make-bump`,
`textures.compare-encodings` and `textures.save`.

## Checks

Run `cargo test -p xrf-app-textures` from the repository root.
