# xrf-editable-object

Reads and writes `.object` editable objects, the source files the SDK's Actor and Level editors open and an OGF is
exported from. `EditableObjectFile` holds the object's flags, meshes, surfaces, and for a dynamic object its bones, bone
parts and motion references. It keeps what an OGF export drops: points shared between faces, the vertex maps their
corners take UVs and weights from, smoothing groups, and each surface's engine shader, compiler shader and game
material by name.

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

Use `read_from_bytes` for owned bytes or `read_from_file` for an open file. `read_from_chunk` reads a file's root,
which must be the body chunk (`0x7777`) alone, and `read_from_body_chunk` reads a body already in hand.

## What is read

A file is one body chunk whose children `CEditableObject::Save` writes in a fixed order. The reader takes each child it
names into a field. How the file laid a container out is that container's `layout`, `xrf-chunk`'s `ChunkLayout`: the
order its chunks were stored in, and every child no field takes, a second copy of a named one included, kept as a
`ChunkPayload`. Content and layout stay apart, so code that builds or edits an object never handles storage detail,
and a container built in code takes `ChunkLayout::default()`.

| Container          | Named chunks                                                                                     |
| ------------------ | ------------------------------------------------------------------------------------------------ |
| Object body        | version, class script, LODs, flags, meshes, surfaces (`SURFACES3`), bones, embedded motions,     |
|                    | motion references (`SMOTIONS3`), bone parts (`BONEPARTS2`), actor transform, description          |
| Mesh (`EditableMesh`) | version, name, bounds, flags, options, points, faces, smoothing groups, vertex map references, |
|                    | surface faces, vertex maps (`VMAPS_2`)                                                           |
| Bone (`EditableBone`) | version, definition, bind pose, the repeated definition, material, shape, joint flags, joint,  |
|                    | joint break, friction, mass                                                                      |

Three layouts older editors saved and the SDK still loads are read: description time stamps 32 or 64 bits wide
(`EditableTimeWidth`, taken from the chunk's size), the faces around each point saved after the points
(`EditableMesh::has_point_faces`), and a flags chunk shorter than 4 bytes, which the loader's `r_chunk` reads into the
low bytes (`EditableObjectFile::flags_size`). The faces around each point are derived data: `get_point_faces` computes
them from the faces, the writer puts down what it computes, and a read refuses stored lists that differ, so an edited
mesh never carries lists its faces no longer give. The older surface, vertex map and bone layouts are kept chunks, and
embedded motions (`EOBJ_CHUNK_SMOTIONS`) are kept as their stored bytes. A field is `Option` wherever the editor's own
loader treats its chunk as optional, so a file that omits a chunk writes back without it.

A bone's collision shape, joint limits and joint kind are [xrf-skeleton](../xrf-skeleton/README.md)'s
`SkeletonBoneShape`, `SkeletonJointLimit` and `SkeletonJointKind`, the `SBoneShape`, `SJointLimit` and `EJointType` an
OGF's IK data stores too; a mesh's bounds are [xrf-math](../xrf-math/README.md)'s `Aabb`.

## Write an object

`write_to_bytes` writes the whole file and `write_to_path` writes it through a staged file, creating the directories
leading to it. The writer follows each container's layout, so a file comes back byte for byte whichever exporter laid
it out: mod-shipped objects store a mesh's vertex map references before its smoothing groups, the other way round from
the SDK. A default layout writes `Save`'s order, and a named chunk added after reading follows the ones the stored
order lists.

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

## Errors and checks

Reads refuse what the SDK refuses or asserts on: an object version other than `0x10`, a mesh version other than
`0x11`, a bone version other than 1 or 2, and a missing chunk the loader requires. They also refuse what would not
come back as it was: a root holding more than the body, a mesh or bone list not numbered from zero, a named chunk
holding more or fewer bytes than its fields, smoothing groups not one per face, a polygon map mark other than 0 or 1,
and a count larger than its chunk could hold. Writes refuse a list longer than its count field holds and a vertex map
whose values, vertex indices and polygon indices disagree on its entry count.

Run `cargo test --locked -p xrf-editable-object` from the repository root. See the
[object file](src/editable_object_file.rs), [meshes](src/mesh/editable_mesh.rs), [bones](src/bone/editable_bone.rs)
and [format tests](src/tests/).
