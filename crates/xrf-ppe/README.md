# xrf-ppe

Reads, writes and evaluates `.ppe` post-process effects, which the engine plays over the view
(`BasicPostProcessAnimator`, `CPostprocessAnimator`). `PpeFile` holds an effect's animated colour and scalar
parameters, with a colour-grading texture and influence channel in version 2. Channels use
[xrf-animation-envelope](../xrf-animation-envelope/README.md). The archive describer and the animation verifier read
effects through it.

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

Use `read_from_bytes` for owned bytes, including archive entries, `read_from_file` for an open file, or `read` for a
reader positioned at the effect's first byte. `XRayByteOrder` selects the little-endian layout.

## Parameters and versions

The file is not a chunk tree: a `u32` version, then a fixed run of parameters with nothing skippable, so a read either
accounts for every byte or fails. `PpeFile` names each parameter after the engine's `pp_params`, in the order the file
stores them:

- `base_color`, `add_color` and `gray_color`: each a `PpeColor` of red, green and blue envelopes
  (`PpeColor::CHANNEL_NAMES`), behind a `base` scalar the engine stores but never reads.
- `gray_value`, `blur`, `duality_horizontal`, `duality_vertical`, `noise_intensity`, `noise_granularity` and
  `noise_fps`: one envelope each.
- `color_map`: the influence envelope and gradient texture version 2 appends.

Version 1 stops after the noise; version 2 carries the colour map, and the two differ in nothing else, so `get_version`
follows from `color_map` rather than being stored apart from it. An empty texture name is allowed;
`PpeColorMap::is_used` checks only whether that name is nonempty.

`BasicPostProcessAnimator::Load` checks neither the version nor the end of the file. It reads any version below 2 as
version 1 and any other as version 2, and ignores bytes after the colour map. This crate refuses both, since writing
such a file back would change its version or drop its tail; no shipped effect has either. Every effect of vanilla,
Clear Sky, CoC, SGM, Anomaly and GAMMA reads except Anomaly's and GAMMA's `fade_in_out.ppe`, which Anomaly's companion
script plays. It is a version 1 body under a
version 2 header, so the engine reads its colour map from whatever memory follows the file. Reading refuses it, naming
the missing colour map.

`get_colors` and `get_values` give the parameters with the names an author knows them by, `get_envelopes` walks every
envelope in stored order, and `get_named_envelopes` names each one (`base color red`, the values, then
`PpeColorMap::INFLUENCE_NAME`). `get_length_seconds` returns the longest first-to-last-key span, or zero when no
channel has a positive span; it does not add a channel's starting time, so an effect whose last key sits at 5.5 s after
a first at 0.5 s lasts 5 s. `get_keys_count` and `is_keyed` include the color-map influence when present.
`PpeColor::get_keys_count` counts all three channels, where the engine's own counts red alone.

## Play an effect

`evaluate(time)` is what the effect does to the screen `time` seconds in, at full strength: a `PpeScreen`, the
engine's `SPPInfo` as `CPostprocessAnimator::Process` fills it. The file's colours are offsets added to
`PpeScreen::IDENTITY` (`pp_identity`: base 0.5, gray 0.333, add 0), a noise parameter carrying no keys keeps the
identity's value (intensity 0, granularity 1, fps 30), and a keyed noise fps plays at a hundred times its stored value.
An effector's factor then fades that screen from the identity, which is how an effect fades in and out: `lerp(factor)`
does it as `SPPInfo::lerp` does. The colours, gray, blur, duality and colour map influence move that far from the
identity's, but the noise is copied at full strength. So the engine stops on a noise granularity at or below zero at any
factor, and `find_refused_noise_granularity` finds the first time the effect plays at, from zero to its length, where
its keyed granularity reaches that.

`get_played_time(elapsed, playback)` is the time an effect plays at, `elapsed` after it starts. It counts from zero,
not from the first key; past `get_length_seconds`, a cyclic effect (`AnimationPlayback::Looped`) starts again by whole
lengths, while one played once (`AnimationPlayback::Once`) runs on until its effector ends that long after it began.

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

The SDK's post-process editor edits keys through `CPostProcessParam`, which an effect mirrors:
- `PpeParameter` names the eleven parameters as the engine numbers them (`pp_params`), and `PpeColorChannel` a
  colour's channel.
- `get_envelope` and `get_envelope_mut` find a parameter's envelope. `get_envelope_mut` gives a version 1 effect a
  colour map, and so version 2, when the influence is asked for.
- `insert_key` sets a key as `add_value` does: through `AnimationEnvelope::insert_key`, then clearing the key's
  tension, continuity and bias.
- `delete_key` takes a key out of every channel of a colour (`delete_value`).
- `get_key_times` lists a colour by its red channel, as the editor's timeline does.

## Write an effect

PPE is a flat sequence of fields. `PpeFile::write` produces the complete file in `writer.buffer` without a chunk header:

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

The writer writes the version the effect's colour map decides rather than raising it to `CURRENT_VERSION` as the
engine's own save does: most shipped effects are version 1, and raising one would append parameters its author never
stored. A supported effect read and written back is byte for byte the file it came from. Envelopes are written as the
shared envelope crate's [encoding](../xrf-animation-envelope/README.md#encoding) describes.

## Errors and checks

Reads reject a file that cannot be opened, unsupported versions, truncated parameters, and trailing bytes. Writes
propagate envelope encoding errors. The crate does not render effects or resolve texture names to assets.

Run `cargo test --locked -p xrf-ppe` from the repository root. See the [effect](src/ppe_file.rs), its
[evaluation](src/ppe_file_evaluation.rs) and [key editing](src/ppe_file_keys.rs), the [parameter](src/ppe_parameter.rs),
the [screen](src/ppe_screen.rs), [colour](src/ppe_color.rs), [colour map](src/ppe_color_map.rs), and
[format tests](src/tests.rs).
