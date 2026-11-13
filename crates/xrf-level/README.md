# xrf-level

Reads compiled X-Ray levels: the files xrLC and the Level Editor leave in a `levels\<name>\` directory.

There is one module per file the game ships, named after the file: `level` is the render bundle, `geom` its geometry,
and the rest are the files beside them. File names are the one vocabulary the engine, the SDK, modders and the data all
share.

This crate reads formats only:

- [xrf-vfs](../xrf-vfs/README.md) finds a level's files across loose trees and archives.
- `xrf-visual` packs what a renderer draws.
- Level visuals are OGF, read through [xrf-ogf](../xrf-ogf/README.md)'s chunk types.
- `level.spawn` is read by [xrf-spawn](../xrf-spawn/README.md) as `LevelSpawnFile`.

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

Every file type has the same entry points, generic over byte order (`xrf_chunk::XRayByteOrder` for every shipped file):

- `read_from_path`, `read_from_file`, `read_from_chunk` over any chunk source;
- for most, `read_from_bytes`, which is how an archived level arrives;
- `FILE_NAME`, the file it reads;
- `write`, filling a `ChunkWriter`'s `buffer`, where a writer exists.

Most files are chunk trees; `level.cform`, `level.ai` and `level.fog_vol` are flat.

## Files

| File                        | Type                 | Reads                                                 | Writer |
| --------------------------- | -------------------- | ----------------------------------------------------- | ------ |
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
| `level.ps_static`           | `LevelPsStaticFile`  | Placed particle effects                               | Yes    |
| `level.snd_static`          | `LevelSndStaticFile` | Placed sounds                                         | Yes    |
| `level.game`                | `LevelGameFile`      | Patrol paths and respawn points                       | Yes    |
| `build.lights`              | `LevelLightsFile`    | The compiler's light list                             | Yes    |
| `level.wallmarks`           | `LevelWallmarksFile` | Editor-only baked decals                              | Yes    |
| `build.prj`                 | `BuildPrjFile`       | The scene the Level Editor hands xrLC                 | Yes    |
| `build.aimap`               | `BuildAimapFile`     | The AI map the Level Editor hands xrAI                | Yes    |
| `build.cform`               | `BuildCformFile`     | The collision form xrLC leaves for xrAI               | Yes    |

`build.prj`, `build.aimap` and `build.cform` are compiler inputs no game loads. The Level Editor writes the first two,
xrLC the third, and xrCompress leaves all three out of archives.

## How a level fits together

A level links its parts by reference:

- A sector names one root visual.
- A hierarchy visual names its children by index in the visuals run.
- A drawable visual has no vertices of its own, only an `OgfGeometryContainerChunk` naming a buffer and range in
  `level.geom` (and, for its fast path, `level.geomx`).

`LevelGeomSource::read_vertices` and `read_indices` serve such a range, refusing one that reaches past its buffer. Every
vertex decodes through `LevelVertexBytes`, so verification and the renderer read the same thing.

## Contracts

### Level bundle and geometry

- `LevelFile` reports what the engine would refuse as data, not as an error: a missing shader table is `shaders: None`,
  the case the renderer stops on. The visuals run is read separately because it is most of the bundle.
- `LevelGeomFile` reads a level's geometry layout without its contents. A median `level.geom` is 45 MB, so buffers are
  skipped, and a declaration misread by one byte fails `assert_read` at the chunk end.
- A vertex size is the furthest any element of stream 0 reaches, not the sum of the elements.
- A visual's optional chunks fail on their own: `LevelVisual::lod`, `swi` and `swi_container` hold the read result, so a
  malformed impostor or window table costs only that part of that visual.

### Collision form and AI map

- The engine casts its file pointer onto the `level.cform` and `level.ai` headers. By default only the header is read:
  36 or 56 bytes from a file of up to 191 MB.
- The `level.ai` GUID is the graph identity a spawn set carries.
- `LevelCformFile::read_with_materials_from_chunk` walks the faces for the `gamemtl.xr` ids they name
  (`LevelCformMaterials`), keeping nothing else, for checking a level against the material library.
- `LevelCformGeometry::build_tracer` puts the whole form in an `xrf-geometry` tracer for the engine's static ray and
  box queries, each face named by its index in the form, whose material the caller reads off the form.
- `LevelAiFile::read_with_nodes_*` reads the nodes as `LevelAiNodes`, packed as stored and decoded one at a time
  (`LevelAiNode`) in any layout the engine loads, versions 8 to 13. Keeping the bytes is what lets the writer
  reproduce the file. `LevelAiNodes::from_nodes` packs computed nodes in a version's layout, refusing a value its bits
  cannot hold; a node's `light` is the bits older compilers left past the links. `LevelAiHeader::get_node_position`
  gives a node's world position as `CLevelGraph` does. `build.aimap`'s nodes keep their cell and height as an
  `xrf-math` `AiNodePosition`.

### Surface materials

`LevelSurfaceMaterials::survey` finds what each shader table entry's surfaces are made of, which the compiled level
does not record.

- Every placed drawable visual's triangles (impostors excluded) are probed with a ray from five centimetres either side
  of each centre, along its normal.
- Their area is tallied per entry against the material found beneath, or as lying on no collision face.
- A render face and the collision face built from it coincide, so the ray finds the one face under it.
- Zaton's 917 entries take two seconds.

### Records and details

- Record chunks the engine casts onto an array (portals, static lights, glows, occlusion meshes) are read through
  `ChunkReader::read_xr_records`, which refuses a chunk that is not whole records.
- `LevelDetailsFile` keeps its slot grid packed as the file's 16-byte words (up to 667,489 slots per level) and
  computes totals by folding over it. `LevelDetailsSlot` is a view built per slot, and its associated functions are the
  one rule every crate packs a slot's words by: a corner's object, its densities, the light (`LevelDetailsLight`, five
  values of four bits, `quantize` as the detail lighter rounds them), and the corners the lighter clears.
- `build.lights` holds one chunk per `LevelLightRole`, rgb, hemi and sun; `get_role_lights` reads one, point and
  directional lights alike (xrLC's `LT_DIRECT` 0 and `LT_POINT` 1).
- A `.dm` is one `CDetail`, so `DetailModel` reads both a library entry and a whole `.dm`.
- Counts read from a file are checked against the bytes remaining before anything is reserved.

## Byte-for-byte writers

Every file with a writer reads and writes back byte for byte. Each was checked against every shipped file in the
reference game trees, and the tests pin the round trip on built fixtures.

Some formats are written in an order their own documentation contradicts; the writers follow the shipped files:

- `level.details` writes objects, grid, then header.
- A patrol path's chunks are 1, 5, 2, 3.
- `build.lights` keeps a chunk that is not whole light records as is.
- `level.env_mod` and `level.ps_static` tell a version chunk from a first record by its size, as the engine does.

Do not reorder a writer to match a format's comments: a file that parses back fine can still differ on every byte.

The compiler inputs were checked against the one sample of each available, Gunslinger's `l01_test`:

- The editor writes its structs whole, so `BuildPrjName` keeps the stale bytes after a name's terminator, and records
  keep their padding and unused `svector` slots.
- `build.prj` writes its smoothing groups after the faces, as `SaveBuild` does.

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

`LevelFile`, `LevelGeomFile` and the body of `level.cform` have no writer; do not assume they preserve bytes.

## Checks

Run `cargo test --locked -p xrf-level` from the repository root.

Sources: the [level bundle](src/level/level_file.rs), [render geometry](src/geom/level_geom_file.rs),
[geometry ranges](src/geom/level_geom_source.rs), [detail library](src/details/level_details_file.rs), and
[public exports](src/lib.rs).
