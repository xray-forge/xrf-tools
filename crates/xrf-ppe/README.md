# xrf-ppe

Reads, writes, evaluates and edits `.ppe` post-process effects, which the engine plays over the view
(`BasicPostProcessAnimator`, `CPostprocessAnimator`).

`PpeFile` holds an effect's animated colour and scalar parameters, plus, in version 2, a colour-grading texture and its
influence channel. Channels are [xrf-animation-envelope](../xrf-animation-envelope/README.md) envelopes. The archive
describer and the animation verifier read effects through it.

## Read an effect

```rust,no_run
use xrf_chunk::XRayByteOrder;
use xrf_ppe::PpeFile;

fn main() -> xrf_error::XrfResult {
  let effect = PpeFile::read_from_path::<XRayByteOrder, _>(&"gamedata/anims/psi.ppe")?;

  println!("version {}, {} keys", effect.get_version(), effect.get_keys_count());
  println!("{} seconds", effect.get_length_seconds());

  for (name, envelope) in effect.get_values() {
    println!("{name}: {} keys", envelope.keys.len());
  }

  if let Some(color_map) = &effect.color_map {
    println!("Color-grading texture: {}", color_map.texture);
  }

  Ok(())
}
```

Use `read_from_bytes` for owned bytes (including archive entries), `read_from_file` for an open file, or `read` for a
reader at the effect's first byte. `XRayByteOrder` selects the little-endian layout.

## Parameters

The file is not a chunk tree: a `u32` version, then a fixed run of parameters. Nothing can be skipped, so a read either
accounts for every byte or fails.

`PpeFile` names each parameter after the engine's `pp_params`, in stored order:

| Field                                               | Contents                                                                                                                          |
| --------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `base_color`, `add_color`, `gray_color`             | A `PpeColor`: red, green and blue envelopes (`PpeColor::CHANNEL_NAMES`), behind a `base` scalar the engine stores but never reads |
| `gray_value`, `blur`                                | One envelope each                                                                                                                 |
| `duality_horizontal`, `duality_vertical`            | One envelope each                                                                                                                 |
| `noise_intensity`, `noise_granularity`, `noise_fps` | One envelope each                                                                                                                 |
| `color_map`                                         | Version 2 only: the influence envelope and gradient texture                                                                       |

- Version 1 ends after the noise; version 2 adds the colour map and differs in nothing else. `get_version` therefore
  follows from `color_map`; it is not stored separately.
- An empty texture name is allowed; `PpeColorMap::is_used` only checks that it is not empty.

Accessors:

- `get_colors` and `get_values` give the parameters under the names authors know them by.
- `get_envelopes` walks every envelope in stored order; `get_named_envelopes` names each (`base color red`, the values,
  then `PpeColorMap::INFLUENCE_NAME`).
- `get_length_seconds` is the longest first-to-last-key span, or zero. It does not add a channel's start time: an
  effect with keys at 0.5 s and 5.5 s lasts 5 s.
- `get_keys_count` and `is_keyed` include the colour map influence. `PpeColor::get_keys_count` counts all three
  channels, where the engine counts red alone.

### Stricter than the engine

`BasicPostProcessAnimator::Load` checks neither the version nor the end of the file: it reads any version below 2 as
1, any other as 2, and ignores bytes after the colour map. This crate refuses both, since writing such a file back would
change its version or drop its tail.

No shipped effect has either problem. Every effect of vanilla, Clear Sky, CoC, SGM, Anomaly and GAMMA reads, except
Anomaly's and GAMMA's `fade_in_out.ppe` (played by Anomaly's companion script). That file is a version 1 body under a
version 2 header, so the engine reads its colour map from whatever memory follows; reading here refuses it, naming the
missing colour map.

## Play an effect

`evaluate(time)` returns what the effect does to the screen `time` seconds in, at full strength: a `PpeScreen`, the
engine's `SPPInfo` as `CPostprocessAnimator::Process` fills it.

- The file's colours are offsets added to `PpeScreen::IDENTITY` (`pp_identity`: base 0.5, gray 0.333, add 0).
- A noise parameter with no keys keeps the identity's value: intensity 0, granularity 1, fps 30.
- A keyed noise fps plays at a hundred times its stored value.

An effector's factor fades that screen from the identity, which is how effects fade in and out. `lerp(factor)` does it
as `SPPInfo::lerp` does:

- colours, gray, blur, duality and colour map influence move that fraction of the way from the identity;
- noise is copied at full strength. So the engine stops on a noise granularity at or below zero at any factor;
  `find_refused_noise_granularity` finds the first time, from zero to the effect's length, at which its keyed
  granularity reaches that.

`get_played_time(elapsed, playback)` is the effect time `elapsed` after it starts. It counts from zero, not from the
first key. Past `get_length_seconds`:

- a cyclic effect (`AnimationPlayback::Looped`) starts again by whole lengths;
- an effect played once (`AnimationPlayback::Once`) runs on until its effector ends.

```rust
use std::time::Duration;

use xrf_animation_envelope::{AnimationEnvelope, AnimationKey, AnimationPlayback, AnimationShape};
use xrf_ppe::{PpeFile, PpeScreen};

let blur = |value: f32, time: f32| AnimationKey { value, time, shape: AnimationShape::Linear, ..Default::default() };
let effect = PpeFile {
  blur: AnimationEnvelope { keys: vec![blur(0.0, 0.0), blur(1.0, 2.0)], ..Default::default() },
  ..PpeFile::default()
};
let time = effect.get_played_time(Duration::from_secs(3), AnimationPlayback::Looped);
let screen: PpeScreen = effect.evaluate(time);

assert_eq!(time, 1.0);
assert_eq!(screen.blur, 0.5);
assert_eq!(screen.base_color, PpeScreen::IDENTITY.base_color);
```

## Edit an effect

Editing mirrors the SDK's post-process editor (`CPostProcessParam`):

- `PpeParameter` names the eleven parameters as the engine numbers them (`pp_params`); `PpeColorChannel` names a
  colour's channel.
- `get_envelope` and `get_envelope_mut` find a parameter's envelope. Asking `get_envelope_mut` for the influence gives a
  version 1 effect a colour map, making it version 2.
- `insert_key` sets a key as `add_value` does: through `AnimationEnvelope::insert_key`, then clearing its tension,
  continuity and bias.
- `delete_key` removes a key from every channel of a colour (`delete_value`).
- `get_key_times` lists a colour's keys by its red channel, as the editor's timeline does.

## Write an effect

`PpeFile::write` produces the complete file in `writer.buffer`, with no chunk header:

```rust
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_ppe::PpeFile;

fn main() -> xrf_error::XrfResult {
  let effect = PpeFile::default();

  let mut writer = ChunkWriter::new();
  effect.write::<XRayByteOrder>(&mut writer)?;
  let restored = PpeFile::read_from_bytes::<XRayByteOrder>(writer.buffer)?;

  assert_eq!(restored, effect);
  assert_eq!(restored.get_version(), PpeFile::FIRST_VERSION);

  Ok(())
}
```

- The version written is the one the colour map implies. The engine's own save raises every effect to
  `CURRENT_VERSION`, but most shipped effects are version 1, and raising one would add parameters its author never
  stored.
- A supported effect read and written back is byte for byte the file it came from.
- Envelopes are written as the envelope crate's [encoding](../xrf-animation-envelope/README.md#encoding) describes.

## Errors

- Reads reject a file that cannot be opened, unsupported versions, truncated parameters, and trailing bytes.
- Writes return envelope encoding errors.

The crate does not render effects or resolve texture names to assets.

## Checks

Run `cargo test --locked -p xrf-ppe` from the repository root.

Sources: the [effect](src/ppe_file.rs), its [evaluation](src/ppe_file_evaluation.rs) and
[key editing](src/ppe_file_keys.rs), the [parameter](src/ppe_parameter.rs), the [screen](src/ppe_screen.rs),
[colour](src/ppe_color.rs), [colour map](src/ppe_color_map.rs), and [format tests](src/tests.rs).
