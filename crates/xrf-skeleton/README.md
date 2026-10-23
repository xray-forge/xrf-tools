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

Both implement `xrf_chunk::ChunkReadWrite`. The parameters chunk reads and writes versions 3 and 4 and refuses others;
only version 4 stores marks, and writing marks as version 3 is refused. The motions chunk refuses a count child that
disagrees with the number of payloads.

A motion's identity is the ordinal pair of its `SkeletonMotionDefinition` and its `SkeletonMotion`: definition `n` plays
payload `n + 1`. The payload's leading `label` is an authoring leftover kept verbatim, and the definition's `motion`
field is kept but not used for pairing; neither is an identity, though `has_label_matching` reports whether a label
still agrees with its definition's name. Pair them through the container's `get_motions` and `get_motion_by_name`.

## Partitions, definitions and marks

`SkeletonPart` is a named partition and its bones as `(name, index)` pairs; `get_bones` lists the names, and the
parameters chunk's `get_bones_count` totals them. The partitions together name every bone a motion's payload keys, in
payload order.

`SkeletonMotionDefinition` holds a motion's name, flags (`esmFX` and the rest), the bone or part it addresses, and its
speed, power, accrue and falloff as stored floats. The engine quantises those four on load and adjusts accrue and
falloff further, so the stored values are not exactly what it plays with. `get_playback_speed` is the speed, or
`DEFAULT_SPEED` (1.0) when the stored one is not positive. `SkeletonMotionMark` is a named run of `(from, to)`
intervals, foot contacts for the most part; its name keeps the line terminator the file used, so a mark is rewritten
byte for byte.

## Decode a motion's keys

```rust
use xrf_chunk::XRayByteOrder;
use xrf_skeleton::{SAMPLE_FPS, SkeletonMotion};

fn main() -> xrf_error::XrfResult {
  // One bone held still: its flags (bit 1) say no rotation stream follows, so one quantised rotation and the bone's
  // offset are all the payload stores.
  let mut remaining: Vec<u8> = Vec::new();

  for component in [0_i16, 0, 0, 32767] {
    remaining.extend_from_slice(&component.to_le_bytes());
  }

  for offset in [0.0_f32, 1.5, 0.0] {
    remaining.extend_from_slice(&offset.to_le_bytes());
  }

  let motion = SkeletonMotion {
    label: "idle".into(),
    count: 45,
    flags: 0b10,
    remaining,
  };
  let bones = motion.decode_bone_motions::<XRayByteOrder>(1)?;

  assert_eq!(motion.get_duration_seconds(), 45.0 / SAMPLE_FPS);
  assert_eq!(bones[0].get_rotation(10).w, 1.0);
  assert_eq!(bones[0].get_translation(10).y, 1.5);

  Ok(())
}
```

`SkeletonMotion` keeps the payload as stored: the label, the declared frame `count`, the first bone's flags byte, and
the remaining key bytes. `decode_bone_motions(bone_count)` dequantises them into one `SkeletonBoneMotion` per bone:
rotations as `Quaternion`s and translations as vectors. A held bone stores one key however many frames the motion
declares, and `get_rotation` and `get_translation` clamp a frame to what a stream holds, so they answer any frame.
Motions sample at a fixed `SAMPLE_FPS` (30), which `get_duration_seconds` divides the frame count by.

The payload stores no bone count: the caller supplies it, usually the partitions' total. Decoding fails when the payload
ends early, when bytes remain after the last bone, which is how a wrong bone count shows, when a keyed stream sits on a
motion declaring no frames, and when a declared count could not fit the remaining bytes, before anything is reserved.

## Checks

Every list read bounds its declared count by the bytes remaining before reserving, and every write refuses a count its
format field cannot hold. The optional `fixtures` feature exports `SkeletonMotionDefinition::new_mock` for dependants'
tests. The `typescript-bindings` feature enables `specta` but no type here derives from it yet.

Run `cargo test --locked -p xrf-skeleton` from the repository root. See the
[parameters chunk](src/skeleton_motion_parameters_chunk.rs), [motions chunk](src/skeleton_motions_chunk.rs),
[definition](src/skeleton_motion_definition.rs), [key decoding](src/skeleton_motion_keys.rs),
[partition](src/skeleton_part.rs), and [mark](src/skeleton_motion_mark.rs).
