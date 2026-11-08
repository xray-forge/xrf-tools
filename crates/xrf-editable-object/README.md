# xrf-editable-object

Reads and writes `.object` files: the editable source the SDK's Actor and Level editors open, and that an OGF is
exported from.

`EditableObjectFile` holds the object's flags, meshes and surfaces, and for a dynamic object its bones, bone parts and
motion references. It keeps what an OGF export drops:

- points shared between faces;
- the vertex maps each face corner takes UVs and weights from;
- smoothing groups;
- each surface's engine shader, compiler shader and game material, by name.

## Read an object

```rust,no_run
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;

fn main() -> xrf_error::XrfResult {
  let object = EditableObjectFile::read_from_path::<XRayByteOrder, _>(&"rawdata/objects/editor/axis.object")?;

  for mesh in object.meshes.iter().flatten() {
    println!("{}: {} points, {} faces", mesh.name, mesh.points.len(), mesh.faces.len());
  }

  for surface in object.surfaces.iter().flatten() {
    println!("{}: {} on {}", surface.name, surface.shader, surface.texture);
  }

  Ok(())
}
```

Other entry points:

- `read_from_bytes` for owned bytes, `read_from_file` for an open file.
- `read_from_chunk` reads a file's root, which must hold the body chunk (`0x7777`) alone.
- `read_from_body_chunk` reads a body already in hand.

## Content and layout

A file is one body chunk whose children `CEditableObject::Save` writes in a fixed order. The reader keeps two things
apart:

- **Content:** each named child is read into a field. A field is `Option` wherever the editor's loader treats its chunk
  as optional, so a file that omits a chunk writes back without it.
- **Layout:** each container's `layout` (`xrf-chunk`'s `ChunkLayout`) records the stored chunk order and keeps every
  child no field takes as a `ChunkPayload`, including a second copy of a named one.

Code that builds or edits an object never handles storage detail. An object built in code uses
`ChunkLayout::default()`.

| Container             | Named chunks                                                                                    |
| --------------------- | ----------------------------------------------------------------------------------------------- |
| Object body           | version, class script, LODs, flags, meshes, surfaces (`SURFACES3`), bones, embedded motions,    |
|                       | motion references (`SMOTIONS3`), bone parts (`BONEPARTS2`), actor transform, description        |
| Mesh (`EditableMesh`) | version, name, bounds, flags, options, points, faces, smoothing groups, corner normals          |
|                       | (`blender-xray`'s `0x1014`, which the SDK does not read), vertex map references, surface faces, |
|                       | vertex maps (`VMAPS_2`)                                                                         |
| Bone (`EditableBone`) | version, definition, bind pose, the repeated definition, material, shape, joint flags, joint,   |
|                       | joint break, friction, mass                                                                     |

Shared types: a bone's collision shape, joint limits and joint kind are [xrf-skeleton](../xrf-skeleton/README.md)'s
`SkeletonBoneShape`, `SkeletonJointLimit` and `SkeletonJointKind` (the engine's `SBoneShape`, `SJointLimit` and
`EJointType`), the same records an OGF's IK data stores. A mesh's
bounds are [xrf-math](../xrf-math/README.md)'s `Aabb`.

### Older layouts

Layouts older editors saved, which the SDK still loads, are read too:

- Description time stamps 32 or 64 bits wide (`EditableTimeWidth`, taken from the chunk's size).
- The faces around each point, saved after the points (`EditableMesh::has_point_faces`).
- A flags chunk shorter than 4 bytes, read into the low bytes as the loader's `r_chunk` does
  (`EditableObjectFile::flags_size`).
- Older surface, vertex map and bone layouts, kept as chunks.

The faces around each point are derived data. `get_point_faces` computes them, the writer writes what it computes, and a
read refuses stored lists that differ, so an edited mesh never carries stale lists.

### Embedded motions

Embedded motions (`EOBJ_CHUNK_SMOTIONS`) are kept as their stored bytes. The SDK reads motion versions 4 to 7 but saves
version 7, so a typed rewrite would change an older object.

- `EditableObjectFile::get_embedded_motions` reads them as `EditableMotion`s, each bone's channels being
  [xrf-animation-envelope](../xrf-animation-envelope/README.md) envelopes.
- `EditableMotion::write_list` writes version 7, for motions built in code.

## Write an object

`write_to_bytes` writes the whole file. `write_to_path` writes it through a staged file and creates the directories
leading to it.

The writer follows each container's layout, so a file comes back byte for byte whichever exporter produced it. For
example, mod-shipped objects store a mesh's vertex map references before its smoothing groups, the reverse of the SDK.
A default layout writes `Save`'s order; a named chunk added after reading goes after the ones the stored order lists.

```rust
use xrf_chunk::{ChunkLayout, XRayByteOrder};
use xrf_editable_object::EditableObjectFile;

fn main() -> xrf_error::XrfResult {
  let object = EditableObjectFile {
    flags: 0,
    flags_size: EditableObjectFile::FLAGS_SIZE,
    class_script: Some(String::new()),
    lods: Some(String::new()),
    meshes: Some(Vec::new()),
    surfaces: Some(Vec::new()),
    bones: None,
    embedded_motions: None,
    motion_references: None,
    bone_parts: None,
    actor_transform: None,
    description: None,
    layout: ChunkLayout::default(),
  };

  let bytes = object.write_to_bytes::<XRayByteOrder>()?;
  let restored = EditableObjectFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  // The object read back knows the order it was stored in; the rest is what was written.
  assert_eq!(restored.layout.order.len(), 6);
  assert_eq!(restored.write_to_bytes::<XRayByteOrder>()?, bytes);

  Ok(())
}
```

## Errors

Reads refuse what the SDK refuses or asserts on:

- an object version other than `0x10`, a mesh version other than `0x11`, a bone version other than 1 or 2;
- a missing chunk the loader requires.

Reads also refuse what would not write back unchanged:

- a root holding more than the body;
- a mesh or bone list not numbered from zero;
- a named chunk with more or fewer bytes than its fields;
- smoothing groups or corner normal triples not one per face;
- a polygon map mark other than 0 or 1;
- a count larger than its chunk could hold.

Writes refuse a list longer than its count field can hold, and a vertex map whose values, vertex indices and polygon
indices disagree on the entry count.

## Checks

Run `cargo test --locked -p xrf-editable-object` from the repository root.

Sources: the [object file](src/editable_object_file.rs), [meshes](src/mesh/editable_mesh.rs),
[bones](src/bone/editable_bone.rs) and [format tests](src/tests/).
