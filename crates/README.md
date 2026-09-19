# XRF Rust crates

Libraries used by the XRF CLI and desktop app. These packages are workspace members and are not published to crates.io.
Package names use hyphens (`xrf-ltx`); Rust imports use underscores (`xrf_ltx`).

## Choose a crate

| Crate                                                      | Purpose                                                                            |
| ---------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| [xrf-animation-envelope](xrf-animation-envelope/README.md) | Read and write scalar animation envelopes shared by ANM and PPE                    |
| [xrf-anm](xrf-anm/README.md)                               | Read and write object and camera motion files                                      |
| [xrf-archive](xrf-archive/README.md)                       | Read `.db` / `.xdb` volume headers, entries, and payloads                          |
| [xrf-archive-stats](xrf-archive-stats/README.md)           | Break an archive subject down by extension, folder, size, volume, and origin       |
| [xrf-build-info](xrf-build-info/README.md)                 | Record and expose binary build provenance                                          |
| [xrf-chunk](xrf-chunk/README.md)                           | Read and write binary chunk framing and fields                                     |
| [xrf-db](xrf-db/README.md)                                 | Parse binary assets: OGF, OMF, spawn, THM, particles, levels, and shader libraries |
| [xrf-dds](xrf-dds/README.md)                               | Decode, encode, and compare DDS textures and mipmaps                               |
| [xrf-dialog](xrf-dialog/README.md)                         | Read dialog trees, source ranges, and translated phrases                           |
| [xrf-dltx](xrf-dltx/README.md)                             | Resolve the Monolith DLTX patch dialect                                            |
| [xrf-error](xrf-error/README.md)                           | Share typed errors and results                                                     |
| [xrf-error-derive](xrf-error-derive/README.md)             | Generate message-based error constructors                                          |
| [xrf-export](xrf-export/README.md)                         | Extract TypeScript extern contracts and render documentation                       |
| [xrf-gamedata](xrf-gamedata/README.md)                     | Validate assembled game data and asset references                                  |
| [xrf-ipc-typescript](xrf-ipc-typescript/README.md)         | Generate the frontend's TypeScript mirror of a Tauri command surface and its types |
| [xrf-job](xrf-job/README.md)                               | Report progress, request cancellation, and plan worker limits                      |
| [xrf-ltx](xrf-ltx/README.md)                               | Parse, resolve, format, and verify LTX configs                                     |
| [xrf-ltx-inspect](xrf-ltx-inspect/README.md)               | Produce config browser records and anchored findings                               |
| [xrf-lua](xrf-lua/README.md)                               | Check LuaJIT syntax and inspect method calls                                       |
| [xrf-lzhuf](xrf-lzhuf/README.md)                           | Compress and decompress X-Ray LZHUF payloads                                       |
| [xrf-material](xrf-material/README.md)                     | Resolve THM dependencies and shader alpha behavior                                 |
| [xrf-output](xrf-output/README.md)                         | Route live messages and ordered worker output                                      |
| [xrf-pack](xrf-pack/README.md)                             | Pack, extract, and patch archive volumes or extract mounted assets                 |
| [xrf-ppe](xrf-ppe/README.md)                               | Read and write animated post-process effects                                       |
| [xrf-report](xrf-report/README.md)                         | Finalize checks and findings into deterministic reports                            |
| [xrf-shaders](xrf-shaders/README.md)                       | Parse shader scripts and resolve source imports                                    |
| [xrf-sound](xrf-sound/README.md)                           | Inspect Ogg/Vorbis metadata and validate audio                                     |
| [xrf-test-utils](xrf-test-utils/README.md)                 | Share test resources and assertions                                                |
| [xrf-texture](xrf-texture/README.md)                       | Build texture recipes, bump pairs, and UI sprite sheets                            |
| [xrf-translation](xrf-translation/README.md)               | Import, edit, build, and verify translations                                       |
| [xrf-typescript](xrf-typescript/README.md)                 | Parse TypeScript and resolve project symbols                                       |
| [xrf-utils](xrf-utils/README.md)                           | Share encoding, file publication, and formatting helpers                           |
| [xrf-vfs](xrf-vfs/README.md)                               | Mount loose files and archives in engine lookup order                              |
| [xrf-visual](xrf-visual/README.md)                         | Prepare OGF geometry and animation data for viewers                                |
| [xrf-xml](xrf-xml/README.md)                               | Parse XML, retain source ranges, and serialize new documents                       |

## Develop

Follow the repository's [build prerequisites](../README.md#build-from-source). From the repository root, replace
`xrf-ltx` below with the package being changed:

```sh
cargo doc --locked -p xrf-ltx --no-deps --open
cargo test --locked -p xrf-ltx
```

Inside this workspace, depend on a library with `xrf-ltx = { workspace = true }`. Crate READMEs point to the public
entry points; generated Rust docs provide signatures and detailed contracts.
See [development commands](../DEVELOPMENT.md)
for workspace checks.
