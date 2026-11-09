# xrf-skeleton

Skeleton data that several formats share, so that no single format crate owns it:

- **Motions.** A skinned OGF can embed its own motions, and an `.omf` file is nothing but a bank of them; both store the
  same two chunks.
- **Bone physics.** A bone's collision shape and joint are stored by both an OGF's IK data and an editable object's
  bone chunks.

This crate has no file type of its own. `xrf-ogf`, [xrf-omf](../xrf-omf/README.md) and
[xrf-editable-object](../xrf-editable-object/README.md) read through it, and [xrf-visual](../xrf-visual/README.md)
decodes and bakes the keys. Its types carry the `Skeleton` prefix, and error text names them that way.

## The two motion chunks

| Type                            | Chunk                   | Contents                                                      |
| ------------------------------- | ----------------------- | ------------------------------------------------------------- |
| `SkeletonMotionParametersChunk` | `OGF_S_SMPARAMS`, ID 15 | A version, the bone partitions, and one definition per motion |
| `SkeletonMotionsChunk`          | `OGF_S_MOTIONS`, ID 14  | A count child (0), then one keyed payload per motion (1 to N) |

Both implement `xrf_chunk::ChunkReadWrite`, following the engine's `motions_value::load`.

Parameters chunk:

- reads and writes every version up to `MAXIMUM_VERSION` (4), refusing newer ones;
- stores definition marks from `SkeletonMotionDefinition::MARKS_VERSION` (4) on; writing marks to an older version is
  refused.

Motions chunk:

- finds each payload by its id, not its position;
- refuses a count child that disagrees with the payloads, a missing payload id, and `MAXIMUM_MOTIONS` (`0x3FFF`) or more
  motions, which a motion id cannot address.

`SkeletonMotionFlag` names the bits of a definition's flags (`ESMFlags`): `mask`, `is_set_in`, `set_in` for the names a
word carries, and `unnamed_in` for bits no name claims.

### Motion identity

A motion is its `SkeletonMotionDefinition` and its `SkeletonMotion`, paired by position: definition `n` plays payload
`n + 1`.

- The payload's leading `label` is an authoring leftover, kept as is. `has_label_matching` reports whether it still
  agrees with its definition's name.
- The definition's `motion` field is kept but not used for pairing.
- Neither is an identity. Pair motions through the container's `get_motions` and `get_motion_by_name`.

## Partitions, definitions and marks

### Partitions

`SkeletonPart` is a named partition and its bones as `(name, index)` pairs. `get_bones` lists the names; the
parameters chunk's `get_bones_count` totals them.

A bone's index is the key run that drives it, whatever order the partitions list it in: `motions_value::load` hands run
`i` to the bone stored with index `i`. So across all partitions each run index appears exactly once. The parameters
chunk refuses, on read and on write:

- a bone keyed by a run past the total;
- a bone keyed by a run another bone already holds;
- more than `MAXIMUM_PARTS` (4) partitions, the fixed array the engine fills.

### Definitions

`SkeletonMotionDefinition` holds a motion's name, flags (`SkeletonMotionFlag`), the bone or part it addresses, and its
speed, power, accrue and falloff as stored floats.

`CMotionDef::Load` keeps only quantized steps of each, so the stored floats are not what the engine plays with. The
engine's values are:

- `get_engine_speed`, `get_engine_power`;
- `get_engine_accrue` and `get_engine_falloff`, widened by `BLEND_RANGE_EXTENSION` (1.5). A cycle's falloff at or above
  its accrue is replaced with one step under it (`is_falloff_replaced`), wrapping round when the accrue is zero.

A declared speed of 1.0 plays at 0.99947. `get_playback_speed` is the engine speed, or `DEFAULT_SPEED` (1.0) where that
comes to zero and the motion would never advance.

### Marks

`SkeletonMotionMark` is a named run of `(from, to)` intervals, mostly foot contacts. Its name keeps the file's line
terminator, so a mark rewrites byte for byte.

## Decode a motion's keys

```rust
use xrf_chunk::XRayByteOrder;
use xrf_skeleton::SkeletonMotion;

fn main() -> xrf_error::XrfResult {
  // One bone held still: its flags (bit 1) say no rotation stream follows, so one quantised rotation and the bone's
  // offset are all its run stores.
  let mut keys: Vec<u8> = vec![0b10];

  for component in [0_i16, 0, 0, 32767] {
    keys.extend_from_slice(&component.to_le_bytes());
  }

  for offset in [0.0_f32, 1.5, 0.0] {
    keys.extend_from_slice(&offset.to_le_bytes());
  }

  let motion = SkeletonMotion {
    label: "idle".into(),
    count: 45,
    keys,
  };
  let bones = motion.decode_bone_motions::<XRayByteOrder>(1)?;

  assert_eq!(motion.get_duration_seconds(), 45.0 / SkeletonMotion::SAMPLE_FPS);
  assert_eq!(bones[0].get_rotation(10).w, 1.0);
  assert_eq!(bones[0].get_translation(10).y, 1.5);

  Ok(())
}
```

`SkeletonMotion` keeps the payload as stored: the label, the declared frame `count`, and the `keys`, one run per bone,
each starting with its own flags byte.

- `decode_bone_motions(bone_count)` dequantises them into one `SkeletonBoneMotion` per run: rotations as `xrf-math`'s
  `Quaternion`s, translations as vectors.
- A held bone stores one key however many frames the motion declares. `get_rotation` and `get_translation` clamp a
  frame to what a stream holds, so they accept any frame.
- Motions sample at a fixed `SkeletonMotion::SAMPLE_FPS` (30); `get_duration_seconds` divides the frame count by it.

The payload stores no bone count; the caller supplies it, usually the partitions' total. Decoding fails:

- when the payload ends early;
- when bytes remain after the last bone (how a wrong bone count shows);
- when a keyed stream belongs to a motion declaring no frames;
- when a declared count could not fit the remaining bytes, checked before anything is reserved.

## A bone's physics

`SkeletonBoneShape` is the engine's `SBoneShape` (112 bytes):

- a `SkeletonBoneShapeKind` saying which shape the bone collides as: none, box, sphere or cylinder;
- flags, with `*_FLAG` constants (`sfNoPickable`, `sfRemoveAfterBreak`, `sfNoPhysics`, `sfNoFogCollider`);
- all three shapes ([xrf-math](../xrf-math/README.md)'s `Obb`, `Sphere` and `Cylinder`). The kind selects one; the
  engine keeps the other two as last edited.

`SkeletonJointKind` is the joint's `EJointType`. `SkeletonJointLimit` is one axis's `SJointLimit`: the range in
radians, `min` to `max`, and how it springs back.

Each kind keeps a number the engine does not name as `Unknown(n)`, so it writes back as stored.

How a joint is stored depends on the format:

- An OGF stores each limit negated, with its ends swapped, for the physics engine, and friction from record version 1
  (`xrf-ogf`'s `OgfJointIkData`).
- An editable object stores the authored limits, with friction and break limits in chunks of their own.

## Limits and fixtures

- Every list read checks its declared count against the bytes remaining before reserving memory.
- Every write refuses a count its format field cannot hold.
- The optional `fixtures` feature exports `SkeletonMotionDefinition::new_mock` for dependants' tests.

## Checks

Run `cargo test --locked -p xrf-skeleton` from the repository root.

The crate is grouped by what it holds: `src/parameters/` and `src/motions/` for the two motion chunks, `src/physics/`
for a bone's physics. Each re-exports its types, and the crate root re-exports them flat.

Sources: the [parameters chunk](src/parameters/skeleton_motion_parameters_chunk.rs),
[motions chunk](src/motions/skeleton_motions_chunk.rs), [definition](src/parameters/skeleton_motion_definition.rs),
[motion and key decoding](src/motions/skeleton_motion.rs), [partition](src/parameters/skeleton_part.rs),
[mark](src/parameters/skeleton_motion_mark.rs), and the [bone shape](src/physics/skeleton_bone_shape.rs).
