# xrf-anm

Reads and writes `.anm` and `.anm1` object motion files, one format under two extensions, which the engine plays a
camera effect or an animated level object from. `AnmFile` holds the motion name, frame range, frame rate, version, and
six animation channels. The level world, the archive describer and the animation verifier read motions through it.

## Read a motion

```rust,no_run
use xrf_anm::{AnmChannel, AnmFile};
use xrf_chunk::XRayByteOrder;

fn main() -> xrf_error::XrfResult {
  let motion = AnmFile::read_from_path::<XRayByteOrder, _>(&"gamedata/anims/camera.anm")?;

  println!("{} frames, {} seconds", motion.get_frame_count(), motion.get_duration_seconds());

  for channel in AnmChannel::ALL {
    println!("{}: {} keys", channel.get_name(), motion.get_channel(channel).keys.len());
  }

  Ok(())
}
```

Use `read_from_bytes` for owned bytes, including an asset read from an archive, or `read_from_file` for an open file.
`read_from_chunk` finds the motion chunk among a reader's children, and `read_from_motion_chunk` reads a motion chunk
already in hand. `XRayByteOrder` selects the little-endian layout.

## Channels and versions

`AnmChannel` is the engine's `EChannelType`: position x, y and z, then rotation heading, pitch and bank. `channels`
holds one envelope per channel in that order, whatever order the file stores them in; index it with
`AnmChannel::get_index`, or ask `get_channel`.

The reader accepts every version `COMotion::Load` reads:

| Version                            | Envelopes                   | Stored channel order     |
| ---------------------------------- | --------------------------- | ------------------------ |
| 3 (`AnmFile::WIDE_VERSION`)        | wide, `CEnvelope::Load_1`   | heading before pitch     |
| 4 (`AnmFile::PITCH_FIRST_VERSION`) | narrow, `CEnvelope::Load_2` | **pitch before heading** |
| 5 (`AnmFile::CURRENT_VERSION`)     | narrow, `CEnvelope::Load_2` | heading before pitch     |

Version 4 ships: `anims\helicopter\aaa.anm` in every workspace tree.

## Write a motion

`AnmFile::write_to_bytes` writes the whole file, the one motion chunk `COMotion::SaveMotion` frames; `write` writes that
chunk's payload alone:

```rust
use xrf_animation_envelope::AnimationEnvelope;
use xrf_anm::AnmFile;
use xrf_chunk::XRayByteOrder;

fn main() -> xrf_error::XrfResult {
  let motion = AnmFile {
    name: "camera".into(),
    frame_start: 0,
    frame_end: 29,
    fps: AnmFile::DEFAULT_FPS,
    version: AnmFile::CURRENT_VERSION,
    channels: Default::default(),
  };

  let bytes = motion.write_to_bytes::<XRayByteOrder>()?;
  let restored = AnmFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(restored, motion);
  assert_eq!(restored.get_duration_seconds(), 1.0);
  assert_eq!(restored.channels, <[AnimationEnvelope; 6]>::default());

  Ok(())
}
```

The writer accepts versions 4 and 5 and preserves the chosen version, storing the channels in that version's own
order, so a version 4 or 5 motion read and written back is byte for byte the file it came from. Version 3 is read-only.
A file is the motion chunk alone: the engine finds that chunk and reads nothing else, so any other chunk a file carries
is not written back. Empty channels are allowed.

## Timing and limits

The frame range includes both endpoints: frames 0 through 29 span 30 frames. Frames are signed, because shipped camera
effects start at -1, and a reversed range counts as zero frames. `get_fps` is `fps`, or `AnmFile::DEFAULT_FPS` (30)
when `fps > 0` is false, and `get_duration_seconds` divides the frame count by it. Envelope key times are seconds; their
first-to-last-key span is separate from the motion's frame-based duration. `get_keys_count` and `is_keyed` count keys
across all six channels; a motion with none is still a well-formed file.

`evaluate(time)` takes seconds and is `COMotion::_Evaluate`: the position from the three position channels and the
rotation as an object animator hands it to `setXYZi`, the pitch in `x`, the heading in `y` and the bank in `z`.
Envelopes and their encoding constraints belong to [xrf-animation-envelope](../xrf-animation-envelope/README.md).

## Play a motion

`get_played_time(elapsed, playback)` is the time in seconds a `CObjectAnimator` plays a motion at, `elapsed` after
`Play`: from the first frame's time, and past the last frame's, `AnmPlayback::Looped` wraps back by whole lengths while
`AnmPlayback::Once` holds there. A one-frame motion holds its first frame either way. Pass the result to `evaluate`:

```rust
use std::time::Duration;

use xrf_anm::{AnmFile, AnmPlayback};

let motion = AnmFile {
  name: String::new(),
  frame_start: 0,
  frame_end: 60,
  fps: 30.0,
  version: AnmFile::CURRENT_VERSION,
  channels: Default::default(),
};
let elapsed = Duration::from_millis(2500);

assert_eq!(motion.get_played_time(elapsed, AnmPlayback::Looped), 0.5);
assert_eq!(motion.get_played_time(elapsed, AnmPlayback::Once), 2.0);
```

Torrid zones and cyclic camera effects loop; a camera effect that is not cyclic plays once, and lives
`get_duration_seconds`, one frame longer than its motion moves, so it ends holding the last frame. A looped time is
wrapped in whole `Duration` lengths before it becomes seconds, so it keeps its pace however long it has played, where
the engine accumulates a frame's step in a float. The engine also evaluates a frame's pose before advancing its time,
a frame behind; that is its update order, which the caller's clock replaces, not part of how a motion plays.

## Errors and checks

Reads reject a file that cannot be opened, missing motion chunks, unsupported versions, truncated payloads, declared key
counts the payload cannot hold, and trailing bytes inside the motion chunk. Writes reject unsupported versions and
propagate envelope encoding errors.

Run `cargo test --locked -p xrf-anm` from the repository root. See the [motion type and operations](src/anm_file.rs),
the [channels](src/anm_channel.rs) and [format tests](src/tests.rs).
