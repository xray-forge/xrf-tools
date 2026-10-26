# xrf-math

The engine's basic math types as the format crates read and write them: `Vector3d` (the engine's `Fvector`),
`Matrix4x4` (its full `Fmatrix`), and the engine's float epsilons. It sits at the bottom of the format crates, depending
only on `xrf-chunk` and `xrf-error`; spawn, level, visual, OGF and particle crates build on it. Both types read and
write through `xrf_chunk::ChunkReadWrite` in the byte order the caller names, and serialize with Serde in camelCase.

## Read, write and place

```rust
use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_math::{Matrix4x4, Vector3d};

fn main() -> xrf_error::XrfResult {
  // Doubles x, then moves by (10, 20, 30): row major, with the translation in the fourth row.
  let matrix: Matrix4x4 = Matrix4x4 {
    values: [
      2.0, 0.0, 0.0, 0.0, //
      0.0, 1.0, 0.0, 0.0, //
      0.0, 0.0, 1.0, 0.0, //
      10.0, 20.0, 30.0, 1.0,
    ],
  };
  let point: Vector3d = Vector3d::new(1.0, 2.0, 3.0);

  assert_eq!(matrix.transform_point(&point), Vector3d::new(12.0, 22.0, 33.0));
  assert_eq!(matrix.transform_direction(&point), Vector3d::new(2.0, 2.0, 3.0));

  let mut writer: ChunkWriter = ChunkWriter::new();

  matrix.write::<XRayByteOrder>(&mut writer)?;
  assert_eq!(writer.bytes_written() as u64, Matrix4x4::SERIALIZED_SIZE);

  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

  assert_eq!(Matrix4x4::read::<XRayByteOrder, _>(&mut reader)?, matrix);

  Ok(())
}
```

## Vector3d

`Vector3d<T = f32>` is three named components; the math is defined for `f32`. It has `new`, the constants `ZERO`, `X`,
`Y` and `Z`, `dot`, `cross`, `length`, `distance` (and their squared forms), `normalize_or_zero` (zero for a zero
vector), the component-wise `min` and `max` a bounding box grows by, `midpoint`, `to_array`, and operators on references
(`&a + &b`, `&a - &b`, `&a * 2.0`, `&a / 2.0`, `-&a`, `sum += &a`). It
converts from `[f32; 3]` and `(f32, f32, f32)`, displays as `x,y,z`, and parses that form back with `FromStr`
(whitespace around each number allowed), failing with `XrfError::Parsing`. On the wire it is three `f32`s, 12 bytes.

## Matrix4x4

`Matrix4x4` is the engine's full `Fmatrix`: sixteen `f32`s, row major, 64 bytes (`ELEMENTS`, `SERIALIZED_SIZE`), read
and written as the engine copies it in and out of a file. The fourth row is the translation (`get_translation`, the
engine's `Fmatrix::c`). `transform_point` applies the translation; `transform_direction` uses only the upper three rows,
so normals and other directions are not moved.

It is not `xrf-spawn`'s `types::Matrix3d`, which is four `Vector3d`s and 48 bytes. Where the engine reads
`sizeof(Fmatrix)`, use `Matrix4x4`: the narrower alias reads 48 of the 64 bytes and can still parse, leaving every
later read misaligned.

## Epsilons

`EPS_S` (`0.0000001`), `EPS` (`0.00001`) and `EPS_L` (`0.001`) are the engine's epsilons from
`xrCore/math_constants.h`, under the engine's names; `EPS_S` is what `fis_zero` compares against by default. Never
redefine one locally: give it a local name that says what it means there (`const GROWTH: f32 = EPS_L`). A value the
engine writes as a literal stays a literal. The UI has the same constants in `@xrf/math`.

## Features and checks

`typescript-bindings` derives Specta types for `Vector3d`. `fixtures` exports `Vector3d::new_mock()`, the vector this
crate's tests use, for dependants' tests.

Run `cargo test --locked -p xrf-math` from the repository root. See [Vector3d](src/vector_3d.rs),
[Matrix4x4](src/matrix_4x4.rs) and the [epsilons](src/math_constants.rs).
