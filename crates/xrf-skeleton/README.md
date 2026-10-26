# xrf-skeleton

Skeleton motion data, shared by the formats that carry it: OGF visuals and OMF libraries. A skinned OGF can embed its
own motions, and an `.omf` file is nothing but a bank of them; both store the same two chunks, so this crate owns their
types and neither format crate does. It owns no file type of its own: `xrf-ogf` and [xrf-omf](../xrf-omf/README.md)
read the chunks through it, and [xrf-visual](../xrf-visual/README.md) decodes and bakes the keys. Types reachable from
the two chunks carry the `Skeleton` prefix, and error text names them that way.

## The two chunks

| Type                            | Chunk                   | Contents                                                      |
| ------------------------------- | ----------------------- | ------------------------------------------------------------- |
| `SkeletonMotionParametersChunk` | `OGF_S_SMPARAMS`, ID 15 | A version, the bone partitions, and one definition per motion |
| `SkeletonMotionsChunk`          | `OGF_S_MOTIONS`, ID 14  | A count child (0), then one keyed payload per motion (1 to N) |

Both implement `xrf_chunk::ChunkReadWrite`, to the rules of the engine's `motions_value::load`. The parameters chunk
reads and writes every version up to `MAXIMUM_VERSION` (4) and refuses newer ones; definitions store marks from
`SkeletonMotionDefinition::MARKS_VERSION` (4) on, and writing marks to an older version is refused. The motions chunk
finds each payload by its id rather than its position, and refuses a count child that disagrees with the number of
payloads, a payload id that is absent, and `MAXIMUM_MOTIONS` (`0x3FFF`) or more motions, which a motion id cannot
address.

`SkeletonMotionFlag` names the bits of a definition's flags, `ESMFlags`: `mask`, `is_set_in`, `set_in` for the names a
word carries, and `unnamed_in` for the bits no name claims.

The crate is grouped by chunk: `src/parameters/` and `src/motions/`, each re-exporting its types and the crate root
re-exporting them flat.

A motion's identity is the ordinal pair of its `SkeletonMotionDefinition` and its `SkeletonMotion`: definition `n` plays
payload `n + 1`. The payload's leading `label` is an authoring leftover kept verbatim, and the definition's `motion`
field is kept but not used for pairing; neither is an identity, though `has_label_matching` reports whether a label
still agrees with its definition's name. Pair them through the container's `get_motions` and `get_motion_by_name`.

## Partitions, definitions and marks

`SkeletonPart` is a named partition and its bones as `(name, index)` pairs; `get_bones` lists the names, and the
parameters chunk's `get_bones_count` totals them. A bone's index is the key run that drives it, whatever order the
partitions list it in: `motions_value::load` hands run `i` to the bone stored with index `i`. So the indices across all
partitions are each run exactly once, and the parameters chunk refuses, on read and on write, a bone keyed by a run past
the total or by one another bone already holds, and more than `MAXIMUM_PARTS` (4), the fixed array the engine fills.

`SkeletonMotionDefinition` holds a motion's name, flags (`SkeletonMotionFlag`), the bone or part it addresses, and its
speed, power, accrue and falloff as stored floats. `CMotionDef::Load` keeps only `quantize`d steps of each, so the
stored floats are not what the engine plays with: `get_engine_speed`, `get_engine_power`, `get_engine_accrue` and
`get_engine_falloff` are, the last two widened by `BLEND_RANGE_EXTENSION` (1.5), and a cycle's falloff at or above its
accrue replaced with one step under it (`is_falloff_replaced`), wrapping round when the accrue is zero. A declared speed
of 1.0 plays at 0.99947. `get_playback_speed` is the engine speed, or `DEFAULT_SPEED` (1.0) where that comes to zero
and the motion would never advance. `SkeletonMotionMark` is a named run of `(from, to)` intervals, foot contacts for the
most part; its name keeps the line terminator the file used, so a mark is rewritten byte for byte.

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
each led by its own flags byte. `decode_bone_motions(bone_count)` dequantises them into one `SkeletonBoneMotion` per
run: rotations as `Quaternion`s and translations as vectors. A held bone stores one key however many frames the motion
declares, and `get_rotation` and `get_translation` clamp a frame to what a stream holds, so they answer any frame.
Motions sample at a fixed `SkeletonMotion::SAMPLE_FPS` (30), which `get_duration_seconds` divides the frame count by.

The payload stores no bone count: the caller supplies it, usually the partitions' total. Decoding fails when the payload
ends early, when bytes remain after the last bone, which is how a wrong bone count shows, when a keyed stream sits on a
motion declaring no frames, and when a declared count could not fit the remaining bytes, before anything is reserved.

## Checks

Every list read bounds its declared count by the bytes remaining before reserving, and every write refuses a count its
format field cannot hold. The optional `fixtures` feature exports `SkeletonMotionDefinition::new_mock` for dependants'
tests.

Run `cargo test --locked -p xrf-skeleton` from the repository root. See the
[parameters chunk](src/parameters/skeleton_motion_parameters_chunk.rs),
[motions chunk](src/motions/skeleton_motions_chunk.rs), [definition](src/parameters/skeleton_motion_definition.rs),
[motion and key decoding](src/motions/skeleton_motion.rs), [partition](src/parameters/skeleton_part.rs), and
[mark](src/parameters/skeleton_motion_mark.rs).
