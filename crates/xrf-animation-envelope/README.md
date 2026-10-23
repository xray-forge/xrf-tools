# xrf-animation-envelope

Reads and writes X-Ray animation envelopes: keys describing how one scalar value changes over time, together with
behaviours for times before and after those keys. A value can represent a position coordinate, a color component, or
an effect's intensity. Both [xrf-anm](../xrf-anm/README.md) and [xrf-ppe](../xrf-ppe/README.md) use these types.

## Write and read an envelope

```rust
use xrf_animation_envelope::{AnimationEnvelope, AnimationKey, AnimationShape};
use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};

fn main() -> xrf_error::XrfResult {
  let step = |value: f32, time: f32| AnimationKey {
    value,
    time,
    shape: AnimationShape::Stepped,
    ..Default::default()
  };
  let envelope = AnimationEnvelope {
    keys: vec![step(0.0, 0.0), step(1.0, 2.0)],
    ..Default::default()
  };

  let mut writer = ChunkWriter::new();
  envelope.write::<XRayByteOrder>(&mut writer)?;

  let mut reader = ChunkReader::from_vec(writer.buffer)?;
  let restored = AnimationEnvelope::read::<XRayByteOrder, _>(&mut reader)?;

  assert_eq!(restored, envelope);
  assert_eq!(restored.get_duration_seconds(), Some(2.0));
  assert_eq!(restored.evaluate(1.0), 0.0);

  Ok(())
}
```

Envelope I/O consumes or emits fields at the current position without a chunk header; the enclosing format owns framing
and decides which encoding to read. Key times and `evaluate`'s argument are seconds.

## Data model

| Type                     | Contents                                                                     |
|--------------------------|------------------------------------------------------------------------------|
| `AnimationEnvelope`      | The behaviour before and after its keys, and the keys                        |
| `AnimationKey`           | Scalar value, time in seconds, shape, and its interpolation parameters       |
| `AnimationInterpolation` | Tension, continuity, bias, and four additional curve parameters              |
| `AnimationShape`         | The curve a span is drawn along, `SHAPE_*`                                   |
| `AnimationBehavior`      | What the envelope does outside its keys, `BEH_*`                             |

The defaults are the engine's constructors: an envelope with no keys, constant either side (`CEnvelope`), and a key of
zeros with the TCB shape (`st_Key`). Shapes and behaviours the engine does not name are kept as `Unknown(raw)`, so a
file written back says what it said; both serialize as their stored `u8`.

`get_duration_seconds` subtracts the first key's time from the last key's time. It returns `None` for no keys and zero
for one key. It neither sorts keys nor finds the minimum and maximum times; keep keys in chronological order when
constructing an envelope.

`evaluate(time)` reads the channel's value at a time as the engine's `evalEnvelope` (`xrCore/Animation/interp.cpp`)
does: zero without keys, the one key's value with one, the keys' curve by each span's shape (TCB, Hermite, Bezier,
line, step, `BEZ2`), and outside the keys the before/after behaviour (reset, constant, repeat, oscillate, offset,
linear). It ports the engine's own arithmetic, quirks included: `range` wraps by `low + time - span * floor(time /
span)`, so a span not starting at zero repeats from where that lands, and oscillation reads the span's length less the
time. An unknown shape reads as the offset alone, as the engine's `default` does; an unknown behaviour leaves the time
outside the keys, and the first or last span carries on where the engine would read past them.

## Encoding

- `read` consumes the narrow layout used by ANM versions 4/5 and PPE. It stores a `u16` key count, byte-sized behaviour
  and shape codes, and quantized interpolation parameters, which a stepped key does not store: those read as zeros.
- `read_wide` consumes the older layout used by ANM version 3. It reads wider counts and codes, and floating-point
  interpolation parameters for every key, stepped or not. Only the low byte of each behaviour and shape code is
  retained.
- `write` emits the narrow layout, as `CEnvelope::Save` does. It refuses more than 65,535 keys and leaves out a stepped
  key's interpolation parameters, judging the shape by the byte it writes. Parameters are quantized over `[-32, 32]`
  and a value outside it is clamped to the nearer end, so arbitrary floating-point parameters need not survive a
  write/read cycle exactly; bytes read and written back do. Wide encoding has no writer, so a version 3 envelope is
  rewritten in the narrow layout.

Reads check the available byte budget before reserving key storage and return errors for truncated data. The enclosing
format must check that its complete payload was consumed.

## Checks

Run `cargo test --locked -p xrf-animation-envelope`, `cargo test --locked -p xrf-anm`, and
`cargo test --locked -p xrf-ppe` from the repository root. The format crates exercise shared envelope encoding,
including quantized parameters, stepped keys, and byte-for-byte round trips.

See the [envelope](src/animation_envelope.rs), its [evaluation](src/animation_envelope_evaluation.rs), the
[key](src/animation_key.rs), its [shape](src/animation_shape.rs), the [behaviour](src/animation_behavior.rs), and the
[interpolation parameters](src/animation_interpolation.rs).
