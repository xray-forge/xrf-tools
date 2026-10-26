# xrf-level

Reads the compiled level of the X-Ray engine: the files xrLC and the level editor leave in a `levels\<name>\` directory,
one module per file the game ships. `level` is the render bundle itself and `geom` its geometry; the rest are the files
beside them, each named for the one it reads. The file name is the only vocabulary the engine, the SDK, the modding
community, and the shipped data all share, so no module groups files under a name of its own.

The crate reads formats and nothing more. Finding a level's files across loose trees and archives is
[xrf-vfs](../xrf-vfs/README.md)'s job; packing what a renderer draws is `xrf-visual`'s. A level visual is an OGF, so
this crate reads the visuals run through [xrf-ogf](../xrf-ogf/README.md)'s chunk types.

## Files and entry points

| File                        | Type                 | Reads                                                | Writer |
| --------------------------- | -------------------- | ---------------------------------------------------- | ------ |
| `level`                     | `LevelFile`          | Header, shader table, sectors, portals, lights, glows | No     |
| `level`                     | `LevelVisualsChunk`  | The visuals run, through `LevelFile::read_visuals_*`  | No     |
| `level.geom`, `level.geomx` | `LevelGeomFile`      | Buffer declarations and counts, slide windows         | No     |
| `level.geom`, `level.geomx` | `LevelGeomSource`    | The same, plus the vertices and indices of a range    | No     |
| `level.cform`               | `LevelCformFile`     | The 36-byte header; geometry or face materials        | Header |
| `level.ai`                  | `LevelAiFile`        | The 56-byte header; with `LevelAiNodes`, every node   | Yes    |
| `level.details`             | `LevelDetailsFile`   | Detail object library and planting grid               | Yes    |
| `.dm`                       | `DetailModel`        | One detail object, also a library entry               | Yes    |
| `level.hom`                 | `LevelHomFile`       | Hierarchical occlusion mesh                           | Yes    |
| `level.som`                 | `LevelSomFile`       | Sound occlusion mesh                                  | Yes    |
| `level.snd_env`             | `LevelSndEnvFile`    | Sound environment volumes                             | Yes    |
| `level.env_mod`             | `LevelEnvModFile`    | Local weather modifiers                               | Yes    |
| `level.fog_vol`             | `LevelFogVolFile`    | Volumetric fog bodies                                 | Yes    |
| `level.ps_static`           | `LevelPsStaticFile`  | Planted particle effects                              | Yes    |
| `level.snd_static`          | `LevelSndStaticFile` | Planted sounds                                        | Yes    |
| `level.game`                | `LevelGameFile`      | Patrol paths and respawn points                       | Yes    |
| `build.lights`              | `LevelLightsFile`    | The compiler's light list                             | Yes    |
| `level.wallmarks`           | `LevelWallmarksFile` | Editor-only baked decals                              | Yes    |

The file types share their doors, generic over the byte order (`xrf_chunk::XRayByteOrder` for every shipped file):
`read_from_path`, `read_from_file`, `read_from_chunk` over any chunk data source, and, for most, `read_from_bytes` for
bytes already in hand, which is how an archived level arrives. Each names the file it reads as `FILE_NAME`. Most files
are chunk trees; `level.cform`, `level.ai` and `level.fog_vol` are flat, which their readers handle. Writers take a
`ChunkWriter` and fill its `buffer`.

Not read here: the SDK's compiler inputs (`build.prj`, `build.cform`, `build.aimap`), which no game loads, and
`level.spawn`, which [xrf-spawn](../xrf-spawn/README.md) reads as `LevelSpawnFile`.

## Read a level

```rust,no_run
use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelFile, LevelGeomFile, LevelVisualsChunk};

# fn main() -> xrf_error::XrfResult {
let directory: PathBuf = PathBuf::from("C:\\Games\\Anomaly\\gamedata\\levels\\jupiter");

let level: LevelFile = LevelFile::read_from_path::<XRayByteOrder, _>(&directory.join("level"))?;
let visuals: Option<LevelVisualsChunk> =
  LevelFile::read_visuals_from_path::<XRayByteOrder, _>(&directory.join("level"))?;
let geometry: LevelGeomFile = LevelGeomFile::read_from_path::<XRayByteOrder, _>(&directory.join("level.geom"))?;

println!(
  "xrLC {}: {} sectors, {} visuals, {} vertices",
  level.header.xrlc_version,
  level.sectors.as_ref().map_or(0, |it| it.sectors.len()),
  visuals.as_ref().map_or(0, |it| it.visuals.len()),
  geometry.get_vertices_count()
);

if level.shaders.is_none() {
  eprintln!("the shader table is absent: the engine refuses to load this level");
}
# Ok(())
# }
```

A level composes by link: a sector names one root visual, a hierarchy visual names its children by index in the visuals
run, and a drawable visual carries no vertices of its own, only an `OgfGeometryContainerChunk` naming a buffer and range
in `level.geom` (and, for its fast path, `level.geomx`). `LevelGeomSource::read_vertices` and `read_indices` serve such
a range, refusing one that reaches past what its buffer declares. Every vertex decodes through one decoder,
`LevelVertexBytes`, so verification and the renderer read the same thing.

## Contracts

- `LevelFile` reports what the engine would refuse as data rather than as an error: an absent shader table is
  `shaders: None`, the case the renderer stops on. The visuals run is read separately because it is the bundle's bulk.
- `LevelGeomFile` keeps a level's geometry to its shape. A median `level.geom` is 45 MB, so buffers are stepped over,
  and a declaration misread by a byte fails `assert_read` at the chunk end instead of answering. A vertex size is the
  furthest an element of stream 0 reaches, not the sum of the elements.
- A visual's optional chunks fail only themselves: `LevelVisual::lod`, `swi` and `swi_container` hold the read result,
  so one malformed impostor or window table costs that visual the part, not the level.
- `level.cform` and `level.ai` are headers the engine casts its file pointer onto; only the header is read by default,
  which is what makes a 36- or 56-byte read off a file of up to 191 MB legal. The `level.ai` guid is the graph identity
  a spawn set carries. `LevelCformFile::read_with_materials_from_chunk` walks the faces for the `gamemtl.xr` ids they
  name (`LevelCformMaterials`), keeping nothing else, which is what checking a level against the material library
  wants.
- `LevelAiFile::read_with_nodes_*` takes the map's nodes as `LevelAiNodes`, kept packed as the file stores them and
  decoded one at a time (`LevelAiNode`) in any layout the engine loads, versions 8 to 13. Keeping the bytes is what
  makes the writer give back the file it read; `LevelAiHeader::get_node_position` turns a node into the world position
  `CLevelGraph` gives it.
- The record chunks the engine casts onto an array - portals, static lights, glows, the occlusion meshes - read through
  `ChunkReader::read_xr_records`, which refuses a chunk that does not divide into whole records.
- `LevelDetailsFile` keeps its slot grid packed as the file's 16-byte words, up to 667,489 slots a level, and answers
  totals by folding over it; `LevelDetailsSlot` is a view built per slot. A `.dm` is one `CDetail`, so `DetailModel`
  reads both a library entry and a whole `.dm`.
- File-derived counts are bounded by the bytes that remain before anything is reserved.

## Byte-for-byte writers

Every companion file with a writer reads and writes back byte for byte: each was checked against every shipped file of
the workspace's game trees, and the crate's tests pin the round trip on built fixtures. Several formats are written in
an order their own documentation contradicts, and the writers follow the shipped files: `level.details` writes
objects, grid, then header; a patrol path's chunks are 1, 5, 2, 3; `build.lights` keeps a chunk that is not whole light
records verbatim. `level.env_mod` and `level.ps_static` tell a version chunk from a first record by its size, as the
engine does. Do not reorder a writer to match a format's comments: a writer that parses back fine can still differ on
every byte.

```rust
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_level::LevelHomFile;

# fn main() -> xrf_error::XrfResult {
let mesh: LevelHomFile = LevelHomFile {
  version: LevelHomFile::CURRENT_VERSION,
  polygons: Vec::new(),
};
let mut writer: ChunkWriter = ChunkWriter::new();

mesh.write::<XRayByteOrder>(&mut writer)?;

assert_eq!(LevelHomFile::read_from_bytes::<XRayByteOrder>(writer.buffer.clone())?, mesh);
# Ok(())
# }
```

`LevelFile`, `LevelGeomFile`, and the body of `level.cform` have no writer: assume they are not byte-faithful.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-level
```

The README is the crate documentation, so its Rust examples run as doctests. See the
[level bundle](src/level/level_file.rs), [render geometry](src/geom/level_geom_file.rs),
[geometry ranges](src/geom/level_geom_source.rs), [detail library](src/details/level_details_file.rs), and
[public exports](src/lib.rs).
