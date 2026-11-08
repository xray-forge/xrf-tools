# xrf-ogf

Reads OGF visuals, the game's model format, and patches them in place without disturbing their other bytes.

An `.ogf` is a chunk tree: a header naming the visual's kind and bounds, then whichever of geometry, texture, bones,
children, kinematics, embedded motions and level-of-detail data that kind carries. Static and skinned models,
hierarchies, progressive meshes, trees and level visuals all share it.

- It builds on `xrf-chunk`, `xrf-math` and `xrf-skeleton`. `xrf-skeleton` owns the motion chunks an OGF shares with an
  `.omf` library (`OGF_S_MOTIONS`, 14, and `OGF_S_SMPARAMS`, 15).
- Users: `xrf-level` (a level's visuals are OGFs read through these chunk types), `xrf-visual` (packs them for the
  renderer), `xrf-gamedata`'s mesh and animation checks, and the CLI's `ogf` commands.

## Read a visual

`OgfFile` (the engine's `FMesh`) reads a whole visual into one value, each chunk an `Option` field. Read it with
`read_from_path`, `read_from_file`, `read_from_bytes` (for an archived visual) or `read_from_chunk`, generic over byte
order (`xrf_chunk::XRayByteOrder` for every shipped file).

```rust
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::{OgfFile, OgfHeaderChunk, OgfModelType};

# fn main() -> xrf_error::XrfResult {
// The smallest visual: a header chunk alone.
let mut writer: ChunkWriter = ChunkWriter::new();

writer.write_xr::<XRayByteOrder, _>(&OgfHeaderChunk {
  version: 4,
  model_type: 0,
  shader_id: 0,
  bounding_box: Aabb {
    min: Vector3d::new(0.0, 0.0, 0.0),
    max: Vector3d::new(1.0, 1.0, 1.0),
  },
  bounding_sphere: Sphere {
    center: Vector3d::new(0.5, 0.5, 0.5),
    radius: 1.0,
  },
})?;

let bytes: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(OgfHeaderChunk::CHUNK_ID)?;
let visual: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(bytes)?;

assert_eq!(OgfModelType::from_raw(visual.header.model_type), Some(OgfModelType::Normal));
assert!(visual.geometry.is_none() && visual.get_motion_names().is_empty());
# Ok(())
# }
```

Only header version 4 is read; any other is `NotImplemented`.

### What is read

`OgfFile` reads every chunk the engine's visual kinds read, in `FMesh.hpp`'s layers:

- header, texture and description, for every visual;
- embedded geometry, or the level buffers and fast path a level visual draws from;
- sliding windows, its own or the level's;
- children, nested or linked;
- a tree's placement, and a LOD's impostor;
- a skeleton's bones, physics, user data, LOD and motions;
- the collision mesh the SDK's exporter adds, which no loader reads.

### Kinds and geometry

- `OgfModelType::from_raw` names the header's kind byte. `OgfModelType::label` spells it as the engine does, showing an
  unknown byte as `MT_UNKNOWN(n)`.
- `OgfGeometry` holds embedded vertices and `u16` indices, decoded by the visual's kind, as the engine does; that is
  what tells overlapping tags apart:
  - a skinned child (`MT_SKELETON_GEOMDEF_*`) as `vertBoned1W` to `vertBoned4W` (`OgfVertexData::Skinned`);
  - any other visual as the Direct3D flexible vertex format its tag names (`OgfFvf`, `OgfVertexData::Static`);
  - a tag neither decodes keeps its value and count, with `vertices: None`.

### Description and references

- A description stores its times as wide as `time_t` was where it was exported, 32 or 64 bits. `OgfDescriptionChunk`
  reads whichever the chunk ends on and writes back the same. If neither fits, `description` is an `Err` and nothing
  else is lost; no loader depends on it.
- `read_motion_refs_from_path` and `read_texture_refs_from_path` read only what a reference check needs.

## Embedded motions

A self-animated visual embeds motion definitions (chunk 15) and their key payloads (chunk 14), paired by position: the
definition at index `n` plays the payload at `n`.

- Use `get_motions`, `get_motion_by_name` and `get_motion_names`. The `label` a payload carries is an authoring
  leftover that may not match its definition's name.
- `get_diverging_labels_count` reports how many labels disagree.
- Payloads without definitions, definitions without payloads, or unequal counts are refused.

## Patch in place

`OgfFile` is an inspection view; there is no whole-file writer. A patch copies every untouched chunk as is and rebuilds
only the changed payload:

- `OgfMotionRefsProcessor::patch_motion_refs_to_path` rewrites the motion references in the kinematics chunk (24, or
  the legacy 19), keeping the chunk id the file used.
- `OgfTextureRefsProcessor::patch_texture_refs_to_path` renames a texture reference in nested child visuals, and refuses
  a run that matches nothing.

Both:

- check the source copies losslessly before writing;
- support a dry run;
- read the written file back, and restore the destination if it does not hold the requested references;
- return an `OgfRefsPatchReport`.

### How the engine reaches chunks

A root walk reaches chunks as the engine does; `OgfChunkAccess` says how:

- **Found** (`find_chunk`): read off the visual's own reader. Its declared size only says where the next chunk starts,
  and its compression mark is ignored.
- **Opened**: bounded by its size, and would be decompressed.

### Residue

`OgfResidue` records where the bytes depart from a well-formed tree without the engine noticing, and
`OgfResidueCause` says why the engine does not see it:

- a fragment too short for a header;
- bytes after the last chunk, headed by an id no loader asks for (text appended to a visual);
- a motion reference past the count the loader reads;
- a last motion refs chunk whose size under- or overstates its references, which end exactly where the file does;
- a last LODs chunk overstating a line that ends within the file.

A compression mark on a found chunk is dropped from its id. A mark on an opened chunk, any other chunk the file cuts
short, and any other departure are refused.

### Normalization

`OgfNormalization::normalize` is the single definition of a well-formed visual, shared by patches and `ogf fix`:

- every root chunk written back as the walk reached it, without a meaningless mark;
- sized to the payload the engine reads;
- a motion refs chunk without entries past its count.

A visual with nothing to fix normalizes to itself byte for byte. `OgfKinematicsChunk::trailing` and
`OgfTextureChunk::trailing` keep bytes a found chunk's size covers beyond what the loader reads, so their own `write`
round-trips them.

`OgfChunksProcessor` walks a visual's chunk tree without interpreting payloads, for surveys and
`find_unknown_chunk_ids`.

## Checks

Run `cargo test --locked -p xrf-ogf` from the repository root. The `fixtures` feature forwards `xrf-skeleton`'s
fixtures for dependants' tests.

Sources: the [visual](src/ogf_file.rs), [motion reference patch](src/patch/ogf_motion_refs_processor.rs),
[texture reference patch](src/patch/ogf_texture_refs_processor.rs), [residue](src/residue/ogf_residue.rs), and
[public exports](src/lib.rs).
