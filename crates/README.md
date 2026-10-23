# XRF Rust crates

Libraries used by the XRF CLI and desktop app. These packages are workspace members and are not published to crates.io.
Package names use hyphens (`xrf-ltx`); Rust imports use underscores (`xrf_ltx`). Each game format has a crate of its
own, and a structure two formats share has one too: `xrf-anm` and `xrf-ppe` both read `xrf-animation-envelope`. Each
README is also its crate's rustdoc page.

## Choose a crate

### Game file formats

| Crate                                                      | Purpose                                                                        |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------ |
| [xrf-chunk](xrf-chunk/README.md)                           | Read and write binary chunk framing and fields, the base of every format crate |
| [xrf-animation-envelope](xrf-animation-envelope/README.md) | Read, write, and evaluate the scalar animation envelopes ANM and PPE share     |
| [xrf-anm](xrf-anm/README.md)                               | Read and write object and camera motions (`.anm`, `.anm1`)                     |
| [xrf-dds](xrf-dds/README.md)                               | Decode, encode, and compare DDS textures and mipmaps                           |
| [xrf-efd](xrf-efd/README.md)                               | Read and write `.efd` evaluation function data                                 |
| [xrf-gamemtl](xrf-gamemtl/README.md)                       | Read and write `gamemtl.xr` game materials and the pairs between them          |
| [xrf-level](xrf-level/README.md)                           | Read compiled levels, one module per file the game ships, and write companions |
| [xrf-light-anim](xrf-light-anim/README.md)                 | Read and write light animation libraries                                       |
| [xrf-lzhuf](xrf-lzhuf/README.md)                           | Compress and decompress X-Ray LZHUF payloads                                   |
| [xrf-ogf](xrf-ogf/README.md)                               | Read OGF visuals and patch them in place                                       |
| [xrf-omf](xrf-omf/README.md)                               | Read and write `.omf` motion libraries                                         |
| [xrf-particles](xrf-particles/README.md)                   | Read and write `particles.xr` effects, groups, and the actions that drive them |
| [xrf-ppe](xrf-ppe/README.md)                               | Read and write animated post-process effects (`.ppe`)                          |
| [xrf-skeleton](xrf-skeleton/README.md)                     | Skeleton motion data that OGF visuals and OMF libraries share                  |
| [xrf-sound](xrf-sound/README.md)                           | Inspect Ogg/Vorbis metadata and validate audio                                 |
| [xrf-spawn](xrf-spawn/README.md)                           | Read and write spawn sets, alife records, graphs, patrols, and artefact points |
| [xrf-thm](xrf-thm/README.md)                               | Read and write `.thm` texture descriptors and their bump pairs                 |

### Archives and the virtual file system

| Crate                                            | Purpose                                                                      |
| ------------------------------------------------ | ---------------------------------------------------------------------------- |
| [xrf-archive](xrf-archive/README.md)             | Read `.db` / `.xdb` volume headers, entries, and payloads                    |
| [xrf-archive-stats](xrf-archive-stats/README.md) | Break an archive subject down by extension, folder, size, volume, and origin |
| [xrf-extension](xrf-extension/README.md)         | Split file extensions and name the ones X-Ray game data ships                |
| [xrf-pack](xrf-pack/README.md)                   | Pack, extract, and patch archive volumes or extract mounted assets           |
| [xrf-vfs](xrf-vfs/README.md)                     | Mount loose files and archives in engine lookup order                        |

### Configs, scripts, and text

| Crate                                            | Purpose                                                                       |
| ------------------------------------------------ | ----------------------------------------------------------------------------- |
| [xrf-dialog](xrf-dialog/README.md)               | Read dialog trees, source ranges, and translated phrases                      |
| [xrf-dltx](xrf-dltx/README.md)                   | Resolve the Monolith DLTX patch dialect                                       |
| [xrf-engine-target](xrf-engine-target/README.md) | Name which engine a game data tree targets: vanilla or extended               |
| [xrf-environment](xrf-environment/README.md)     | Read weather, suns, thunderbolts and ambients as the target engine reads them |
| [xrf-export](xrf-export/README.md)               | Extract TypeScript extern contracts and render documentation                  |
| [xrf-ltx](xrf-ltx/README.md)                     | Parse, resolve, format, and verify LTX configs                                |
| [xrf-ltx-inspect](xrf-ltx-inspect/README.md)     | Produce config browser records and anchored findings                          |
| [xrf-lua](xrf-lua/README.md)                     | Check LuaJIT syntax and inspect method calls                                  |
| [xrf-shaders](xrf-shaders/README.md)             | Parse shader scripts and resolve source imports                               |
| [xrf-translation](xrf-translation/README.md)     | Import, edit, build, and verify translations                                  |
| [xrf-typescript](xrf-typescript/README.md)       | Parse TypeScript and resolve project symbols                                  |
| [xrf-xml](xrf-xml/README.md)                     | Parse XML, retain source ranges, and serialize new documents                  |

### Game data, assets, and verification

| Crate                                  | Purpose                                                 |
| -------------------------------------- | ------------------------------------------------------- |
| [xrf-gamedata](xrf-gamedata/README.md) | Validate assembled game data and asset references       |
| [xrf-material](xrf-material/README.md) | Resolve THM dependencies and shader alpha behavior      |
| [xrf-texture](xrf-texture/README.md)   | Build texture recipes, bump pairs, and UI sprite sheets |
| [xrf-visual](xrf-visual/README.md)     | Prepare OGF geometry and animation data for viewers     |

### Rendering

| Crate                                                | Purpose                                                                    |
| ---------------------------------------------------- | -------------------------------------------------------------------------- |
| [xrf-renderer](xrf-renderer/README.md)               | Draw viewports into the application's windows on one GPU and render thread |
| [xrf-renderer-core](xrf-renderer-core/README.md)     | The frame graph and wgpu-level core a renderer builds on, format-agnostic  |
| [xrf-renderer-derive](xrf-renderer-derive/README.md) | The derives `xrf-renderer-core` re-exports                                 |
| [xrf-world](xrf-world/README.md)                     | Run what moves in a level each frame and hand it to the renderer           |
| [xrf-dlss](xrf-dlss/README.md)                       | NVIDIA DLSS for the renderer's views; without the `sdk` feature, none      |

### Frontend bindings

| Crate                                              | Purpose                                                                            |
| -------------------------------------------------- | ---------------------------------------------------------------------------------- |
| [xrf-ipc](xrf-ipc/README.md)                       | Mark a constant for the frontend's generated bindings where it is declared         |
| [xrf-ipc-derive](xrf-ipc-derive/README.md)         | The `#[constant]` attribute `xrf-ipc` re-exports                                   |
| [xrf-ipc-typescript](xrf-ipc-typescript/README.md) | Generate the frontend's TypeScript mirror of a Tauri command surface and its types |

### Shared infrastructure

| Crate                                          | Purpose                                                           |
| ---------------------------------------------- | ----------------------------------------------------------------- |
| [xrf-build-info](xrf-build-info/README.md)     | Record and expose binary build provenance                         |
| [xrf-env-flags](xrf-env-flags/README.md)       | Declare and read every environment variable the workspace honours |
| [xrf-error](xrf-error/README.md)               | Share typed errors and results                                    |
| [xrf-error-derive](xrf-error-derive/README.md) | Generate message-based error constructors                         |
| [xrf-job](xrf-job/README.md)                   | Report progress, request cancellation, and plan worker limits     |
| [xrf-math](xrf-math/README.md)                 | Vectors and matrices in the engine's layout                       |
| [xrf-output](xrf-output/README.md)             | Route live messages and ordered worker output                     |
| [xrf-report](xrf-report/README.md)             | Finalize checks and findings into deterministic reports           |
| [xrf-test-utils](xrf-test-utils/README.md)     | Share test resources and assertions                               |
| [xrf-utils](xrf-utils/README.md)               | Share encoding, file publication, and formatting helpers          |

## Develop

Follow the repository's [build prerequisites](../README.md#build-from-source). From the repository root, replace
`xrf-ltx` below with the package being changed:

```sh
cargo doc --locked -p xrf-ltx --no-deps --open
cargo test --locked -p xrf-ltx
```

Inside this workspace, depend on a library with `xrf-ltx = { workspace = true }`. Crate READMEs point to the public
entry points; generated Rust docs provide signatures and detailed contracts. See
[development commands](../DEVELOPMENT.md) for workspace checks.
