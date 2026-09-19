# xrf-ppe

Reads and writes `.ppe` post-process effects. `PpeFile` holds animated color and scalar parameters, with a color-grading
texture and influence channel in version 2. Channels use [xrf-animation-envelope](../xrf-animation-envelope/README.md).

## Read an effect

```rust,no_run
use xrf_chunk::XRayByteOrder;
use xrf_ppe::{PPE_VALUES, PpeFile};

fn main() -> xrf_error::XrfResult {
  let effect = PpeFile::read_from_path::<XRayByteOrder, _>(&"gamedata/anims/psy.ppe")?;

  println!("{} keys, {} seconds", effect.get_keys_count(), effect.get_length_seconds());

  for (name, envelope) in PPE_VALUES.iter().zip(&effect.values) {
    println!("{name}: {} keys", envelope.keys.len());
  }

  if let Some(color_map) = &effect.color_map {
    println!("Color-grading texture: {}", color_map.texture);
  }

  Ok(())
}
```

Use `read_from_bytes` for owned bytes, including archive entries, or `read_from_file` for an open file.
`XRayByteOrder` selects the little-endian layout. Versions 1 and 2 are supported for both reading and writing.

## Parameters and versions

`colors` follows `PPE_COLORS`: base color, add color, and gray color. Each `PpeColor` stores a base scalar and red,
green, and blue envelopes. `values` follows `PPE_VALUES`: gray value, blur, horizontal and vertical duality, noise
intensity, noise granularity, and noise FPS.

Version 1 contains those three colors and seven scalar channels. Version 2 additionally requires `PpeColorMap`, which
holds an influence envelope and texture name. An empty texture name is allowed; `PpeColorMap::is_used` checks only
whether that name is nonempty.

`get_envelopes` traverses the color channels, scalar channels, and optional color-map influence in stored order.
`get_length_seconds` returns the longest first-to-last-key span, or zero when no channel has a positive span. It does
not add a channel's starting time. `get_keys_count` and `is_keyed` include the color-map influence when present.

## Write an effect

PPE is a flat sequence of fields. `PpeFile::write` produces the complete file in `writer.buffer` without a chunk header:

```rust
use xrf_animation_envelope::AnimationEnvelope;
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_ppe::{PpeColor, PpeFile};

fn main() -> xrf_error::XrfResult {
  let empty = AnimationEnvelope { behavior: (1, 1), keys: vec![] };
  let color = PpeColor {
    base: 0.0,
    red: empty.clone(),
    green: empty.clone(),
    blue: empty.clone(),
  };
  let effect = PpeFile {
    version: 1,
    colors: vec![color; PpeFile::COLOR_COUNT],
    values: vec![empty; PpeFile::VALUE_COUNT],
    color_map: None,
  };

  let mut writer = ChunkWriter::new();
  effect.write::<XRayByteOrder>(&mut writer)?;
  let restored = PpeFile::read_from_bytes::<XRayByteOrder>(writer.buffer)?;

  assert_eq!(restored, effect);
  assert!(!restored.is_keyed());

  Ok(())
}
```

The writer preserves `version`. It requires exactly three colors and seven scalar channels, forbids a color map in
version 1, and requires one in version 2, even when its texture name is empty. Follow the shared envelope crate's
[encoding obligations](../xrf-animation-envelope/README.md#encoding-and-caller-obligations) when constructing keys.

## Errors and checks

Reads reject unsupported versions, truncated parameters, and trailing bytes. Writes reject incompatible versions,
parameter counts, and color-map presence, and propagate envelope encoding errors. The crate does not render effects,
evaluate curves, or resolve texture names to assets.

Run `cargo test --locked -p xrf-ppe` from the repository root. See
the [effect](src/ppe_file.rs), [color](src/ppe_color.rs), [color map](src/ppe_color_map.rs), and
[format tests](src/tests.rs).
