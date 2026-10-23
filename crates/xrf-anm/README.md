# xrf-anm

Reads and writes `.anm` and `.anm1` object motion files, one format under two extensions, which the engine plays a
camera effect or an animated level object from. `AnmFile` holds the motion name, frame range, frame rate, version, and
six animation channels. The level world, the archive describer and the animation verifier read motions through it.

## Read a motion

```rust,no_run
use xrf_anm::{ANM_CHANNELS, AnmFile};
use xrf_chunk::XRayByteOrder;

fn main() -> xrf_error::XrfResult {
  let motion = AnmFile::read_from_path::<XRayByteOrder, _>(&"gamedata/anims/camera.anm")?;

  println!("{} frames, {} seconds", motion.get_frame_count(), motion.get_duration_seconds());

  for (name, channel) in ANM_CHANNELS.iter().zip(&motion.channels) {
    println!("{name}: {} keys", channel.keys.len());
  }

  Ok(())
}
```

Use `read_from_bytes` for owned bytes, including an asset read from an archive, or `read_from_file` for an open file.
`read_from_chunk` finds the motion chunk among a reader's children, and `read_from_motion_chunk` reads a motion chunk
already in hand. `XRayByteOrder` selects the little-endian layout. The reader accepts versions 3, 4, and 5; version 3
uses the older, wider envelope encoding.

## Write a motion

`AnmFile::write` writes the motion payload. Frame it with `AnmFile::CHUNK_ID` to produce a complete file:

```rust
use xrf_animation_envelope::AnimationEnvelope;
use xrf_anm::{ANM_DEFAULT_FPS, AnmFile};
use xrf_chunk::{ChunkWriter, XRayByteOrder};

fn main() -> xrf_error::XrfResult {
  let motion = AnmFile {
    name: "camera".into(),
    frame_start: 0,
    frame_end: 29,
    fps: ANM_DEFAULT_FPS,
    version: AnmFile::CURRENT_VERSION,
    channels: vec![AnimationEnvelope::default(); AnmFile::CHANNEL_COUNT],
  };

  let mut writer = ChunkWriter::new();
  motion.write::<XRayByteOrder>(&mut writer)?;
  let bytes = writer.flush_chunk_into_buffer::<XRayByteOrder>(AnmFile::CHUNK_ID)?;

  let restored = AnmFile::read_from_bytes::<XRayByteOrder>(bytes)?;
  assert_eq!(restored, motion);
  assert_eq!(restored.get_duration_seconds(), 1.0);

  Ok(())
}
```

The writer accepts versions 4 and 5, which lay their envelopes out identically, and preserves the chosen version, so a
version 4 or 5 motion read and written back is byte for byte the file it came from. Version 3 is read-only. Writing
requires exactly six channels in `ANM_CHANNELS` order: position x/y/z, then rotation heading/pitch/bank. Empty channels
are allowed.

## Timing and limits

The frame range includes both endpoints: frames 0 through 29 span 30 frames. Frames are signed, because shipped camera
effects start at -1, and a reversed range counts as zero frames. `get_fps` is `fps`, or `ANM_DEFAULT_FPS` (30) when
`fps > 0` is false, and `get_duration_seconds` divides the frame count by it. Envelope key times are seconds; their
first-to-last-key span is separate from the motion's frame-based duration. `get_keys_count` and `is_keyed` count keys
across all six channels; a motion with none is still a well-formed file.

`evaluate(time)` takes seconds and is `COMotion::_Evaluate`: the position from channels 0 to 2 and the rotation as an
object animator hands it to `setXYZi`, the pitch (channel 4) in `x`, the heading (channel 3) in `y` and the bank in
`z`. A missing channel reads as zero. `get_looped_time(elapsed)` is the time in seconds a looping `CObjectAnimator`
plays at: from the first frame's time, wrapped back by whole lengths past the last frame's; a one-frame motion holds
its first frame. Envelopes and their encoding constraints belong to
[xrf-animation-envelope](../xrf-animation-envelope/README.md).

## Errors and checks

Reads reject a file that cannot be opened, missing motion chunks, unsupported versions, truncated payloads, declared key
counts the payload cannot hold, and trailing bytes inside the motion chunk. Writes reject unsupported versions and
incorrect channel counts, and propagate envelope encoding errors.

Run `cargo test --locked -p xrf-anm` from the repository root. See the [motion type and operations](src/anm_file.rs)
and [format tests](src/tests.rs).
