# xrf-animation-envelope

Reads and writes X-Ray animation envelopes: keys describing how one scalar value changes over time, together with
behavior codes for times before and after those keys. A value can represent a position coordinate, a color component, or
an effect's intensity. Both [xrf-anm](../xrf-anm/README.md) and [xrf-ppe](../xrf-ppe/README.md) use these types.

## Write and read an envelope

```rust
use xrf_animation_envelope::{AnimationEnvelope, AnimationKey};
use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};

fn main() -> xrf_error::XrfResult {
    let envelope = AnimationEnvelope {
        behavior: (1, 1),
        keys: vec![
            AnimationKey { value: 0.0, time: 0.0, shape: 4, interpolation: None },
            AnimationKey { value: 1.0, time: 2.0, shape: 4, interpolation: None },
        ],
    };

    let mut writer = ChunkWriter::new();
    envelope.write::<XRayByteOrder>(&mut writer)?;

    let mut reader = ChunkReader::from_vec(writer.buffer)?;
    let restored = AnimationEnvelope::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(restored, envelope);
    assert_eq!(restored.get_duration_seconds(), Some(2.0));

    Ok(())
}
```

The example uses stepped keys (`shape = 4`), which carry no interpolation parameters in the narrow encoding. Envelope
I/O consumes or emits fields at the current position without a chunk header; the enclosing format owns framing and
decides which encoding to read.

## Data model

| Type                     | Contents                                                                         |
|--------------------------|----------------------------------------------------------------------------------|
| `AnimationEnvelope`      | Before/after behavior codes and a vector of keys                                 |
| `AnimationKey`           | Scalar value, time in seconds, shape code, and optional interpolation parameters |
| `AnimationInterpolation` | Tension, continuity, bias, and four additional curve parameters                  |

`get_duration_seconds` subtracts the first key's time from the last key's time. It returns `None` for no keys and zero
for one key. It neither sorts keys nor finds the minimum and maximum times; keep keys in chronological order when
constructing an envelope.

Behavior and shape codes are stored as numbers. This crate does not evaluate curves, apply before/after behavior, or
validate that those codes are supported by a player.

## Encoding and caller obligations

- `read` consumes the narrow layout used by ANM versions 4/5 and PPE. It stores a `u16` key count, byte-sized behavior
  and shape codes, and quantized interpolation parameters.
- `read_wide` consumes the older layout used by ANM version 3. It reads wider counts and codes, and floating-point
  interpolation parameters. Only the low byte of each behavior and shape code is retained.
- `write` emits the narrow layout. It refuses more than 65,535 keys. Interpolation parameters are quantized over
  `[-32, 32]`, so arbitrary floating-point parameters need not survive a write/read cycle exactly.

When constructing keys for writing, use `interpolation: None` for shape 4 and `Some(AnimationInterpolation { ... })`
for other shapes. The writer follows the option as supplied and does not check its consistency with the shape. Wide
reads retain interpolation parameters even for stepped keys, so writing those keys in the narrow layout requires
normalizing that field first; wide encoding has no writer here.

Reads check the available byte budget before reserving key storage and return errors for truncated data. The enclosing
format must check that its complete payload was consumed.

## Checks

Run `cargo test --locked -p xrf-animation-envelope`, `cargo test --locked -p xrf-anm`, and
`cargo test --locked -p xrf-ppe` from the repository root. The format crates exercise shared envelope encoding,
including quantized parameters, stepped keys, and byte-for-byte round trips.

See the [envelope](src/animation_envelope.rs), [key](src/animation_key.rs), and
[interpolation parameters](src/animation_interpolation.rs).
