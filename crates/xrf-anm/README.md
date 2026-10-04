# xrf-anm

Reads and writes `.anm` and `.anm1` object motion files used for cameras and animated objects. `AnmFile` holds the
motion name, frame range, frame rate, version, and six animation channels.

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
`XRayByteOrder` selects the little-endian layout. The reader accepts versions 3, 4, and 5; version 3 uses the older,
wider envelope encoding.

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
    channels: vec![AnimationEnvelope { behavior: (1, 1), keys: vec![] }; AnmFile::CHANNEL_COUNT],
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

The writer accepts versions 4 and 5 and preserves the chosen version. Version 3 is read-only. Writing requires exactly
six channels in `ANM_CHANNELS` order: position x/y/z, then rotation heading/pitch/bank. Empty channels are allowed.

## Timing and limits

The frame range includes both endpoints: frames 0 through 29 span 30 frames. A reversed range counts as zero frames.
`get_duration_seconds` divides the frame count by `fps`, using `ANM_DEFAULT_FPS` (30) when `fps > 0` is false. Envelope
key times are seconds; their first-to-last-key span is separate from the motion's frame-based duration.

`evaluate(time)` is `COMotion::_Evaluate`: the position from channels 0 to 2 and the rotation as an object animator
hands it to `setXYZi`, the pitch (channel 4) in `x`, the heading (channel 3) in `y` and the bank in `z`.
`get_looped_time(elapsed)` is the time a looping `CObjectAnimator` plays at: from the first frame's time, wrapped back
by whole lengths past the last frame's. Envelopes and their encoding constraints belong to
[xrf-animation-envelope](../xrf-animation-envelope/README.md).

## Errors and checks

Reads reject missing motion chunks, unsupported versions, truncated payloads, and trailing bytes inside the motion
chunk. Writes reject unsupported versions and incorrect channel counts, and propagate envelope encoding errors.

Run `cargo test --locked -p xrf-anm` from the repository root. See the [motion type and operations](src/anm_file.rs)
and [format tests](src/tests.rs).
