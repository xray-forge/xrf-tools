# xrf-ogf

Reads the OGF visual format, what a model file carries, and patches that rewrite one in place. An `.ogf` is a chunk
tree: a header naming the visual's kind and bounds, then whichever of geometry, texture, bones, children, kinematics,
embedded motions and level-of-detail data that kind carries. Static and skinned models, hierarchies, progressive meshes,
trees and level visuals all share it.

The crate sits on `xrf-chunk`, `xrf-math` and `xrf-skeleton`, which owns the motion chunks an OGF and an `.omf` motion
library share (`OGF_S_MOTIONS`, 14, and `OGF_S_SMPARAMS`, 15). Consumers are `xrf-level`, whose visuals run is a list of
OGFs read through this crate's chunk types, `xrf-visual`, which packs them for the renderer, `xrf-gamedata`'s mesh and animation checks, and the CLI's `ogf`
commands.

## Read a visual

`OgfFile` (`FMesh` in the engine) reads a whole visual into one value, each chunk an `Option` field. It has
`read_from_path`, `read_from_file`, `read_from_bytes` for a visual served out of an archive, and `read_from_chunk`,
generic over the byte order (`xrf_chunk::XRayByteOrder` for every shipped file).

```rust
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_math::Vector3d;
use xrf_ogf::{OgfBox, OgfFile, OgfHeaderChunk, OgfModelType, OgfSphere};

# fn main() -> xrf_error::XrfResult {
// The smallest visual: a header chunk alone.
let mut writer: ChunkWriter = ChunkWriter::new();

writer.write_xr::<XRayByteOrder, _>(&OgfHeaderChunk {
  version: 4,
  model_type: 0,
  shader_id: 0,
  bounding_box: OgfBox {
    min: Vector3d::new(0.0, 0.0, 0.0),
    max: Vector3d::new(1.0, 1.0, 1.0),
  },
  bounding_sphere: OgfSphere {
    position: Vector3d::new(0.5, 0.5, 0.5),
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

- `OgfFile` reads every chunk the engine's visual kinds read, layer by layer as `FMesh.hpp` numbers them: header,
  texture and description for every visual; embedded geometry, or the level buffers and fast path a level visual draws
  from; sliding windows, own or the level's; children nested or linked; a tree's placement; a LOD's impostor; a
  skeleton's bones, physics, user data, LOD and motions; and the collision mesh the SDK's exporter adds, which no loader
  reads. Only header version 4 is read; any other is `NotImplemented`.
- `OgfModelType::from_raw` names the header's raw kind byte, and `OgfModelType::label` spells it as the engine does,
  keeping an unknown byte visible as `MT_UNKNOWN(n)`.
- `OgfGeometry` holds embedded vertices and `u16` indices, decoded as the visual's kind decides, which is what the
  engine does and what tells overlapping tags apart: a skinned child's (`MT_SKELETON_GEOMDEF_*`) as `vertBoned1W` to
  `vertBoned4W` (`OgfVertexData::Skinned`), any other visual's as the Direct3D flexible vertex format its tag is
  (`OgfFvf`, `OgfVertexData::Static`). A tag neither decodes keeps its value and count with `vertices: None`.
- A description stores its times as `time_t` was wide where it was exported, 32 or 64 bits; `OgfDescriptionChunk` reads
  whichever the chunk ends on and writes back the same. One neither reads leaves `description` an `Err`, costing the
  visual nothing else, since no loader depends on it.
- `read_motion_refs_from_path` and `read_texture_refs_from_path` read only what a reference check needs.

## Embedded motions

A self-animated visual embeds motion definitions (chunk 15) and their key payloads (chunk 14). They pair by ordinal: the
definition at index `n` plays the payload at `n`. The label a payload carries is an authoring leftover that may not
match its definition's name. Use `get_motions`, `get_motion_by_name` and `get_motion_names` rather than the preserved
`label` or `motion` fields; `get_diverging_labels_count` reports how many labels disagree. A visual with payloads and
no definitions, definitions and no payloads, or unequal counts is refused.

## Lossless patches

`OgfFile` is an inspection view with no whole-file writer: parsed values are not a document. A narrow edit copies every
untouched chunk verbatim and rebuilds only the changed payload:

- `OgfMotionRefsProcessor::patch_motion_refs_to_path` rewrites the motion references of the kinematics chunk (24, or
  the legacy 19), keeping the chunk id the file used.
- `OgfTextureRefsProcessor::patch_texture_refs_to_path` renames a texture reference in the nested child visuals, and
  refuses a run that matches nothing.

Both check that the source copies losslessly before writing, support a dry run, read the written file back, and restore
the destination when the result does not hold the requested references. Each returns an `OgfRefsPatchReport`.

A root walk reaches chunks as the engine does, and `OgfChunkAccess` says how: a chunk the loader finds
(`find_chunk`) is read off the visual's own reader, its declared size only saying where the next one starts and its
compression mark ignored; a chunk it opens is bounded and would be decompressed. `OgfResidue` is where the bytes depart
from a well-formed tree without the engine noticing, and `OgfResidueCause` says why that is invisible to it: a fragment
too short for a header, a motion reference cut past the count the loader reads, or a last motion refs chunk whose size
understates or overstates the references it holds, which end exactly where the file does. A mark on a found chunk is
dropped from its id; one on an opened chunk, any other chunk the file cuts short, and any other departure are refused.

`OgfNormalization::normalize` is the one definition of a well-formed visual, so a patch and `ogf fix` agree on it: every
root chunk written back as the walk reached it, without a meaningless mark, sized to the payload the engine reads, and
a motion refs chunk without what its count never reaches. A visual with nothing to correct normalizes to itself byte
for byte. `OgfKinematicsChunk::trailing` and `OgfTextureChunk::trailing` keep the bytes a found chunk's size covers past
what the loader reads, so their own `write` round trips them.

`OgfChunksProcessor` walks a visual's chunk tree without interpreting payloads, for surveys and
`find_unknown_chunk_ids`.

## Checks

From the repository root:

```sh
cargo test --locked -p xrf-ogf
```

The README is the crate documentation, so its Rust examples run as doctests. The `fixtures` feature forwards
`xrf-skeleton`'s fixtures for dependants' tests. See the [visual](src/ogf_file.rs),
[motion reference patch](src/ogf_motion_refs_processor.rs),
[texture reference patch](src/ogf_texture_refs_processor.rs), [residue](src/residue/ogf_residue.rs), and
[public exports](src/lib.rs).
