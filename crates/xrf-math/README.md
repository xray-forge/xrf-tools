# xrf-math

The engine's basic math types, as the format crates read and write them.

| Type                                | Engine                                 | Bytes                 |
| ----------------------------------- | -------------------------------------- | --------------------- |
| `Vector2d`                          | `Fvector2`                             | 8                     |
| `Vector3d`                          | `Fvector`                              | 12                    |
| `Matrix4x4`                         | `Fmatrix`                              | 64                    |
| `Affine3d`                          | `Fmatrix`, without its fourth column   | 48                    |
| `Quaternion`                        | `Fquaternion`                          | —                     |
| `Color`                             | `Fcolor`                               | 16                    |
| `Aabb`, `Obb`, `Sphere`, `Cylinder` | `Fbox`, `Fobb`, `Fsphere`, `Fcylinder` | see [Shapes](#shapes) |
| `AiNodePosition`                    | `SNodePositionOld`                     | 6                     |

It also holds the engine's float epsilons and the display-colour helpers.

- It depends only on `xrf-chunk` and `xrf-error`. The spawn, level, skeleton, visual, OGF and particle crates build on
  it.
- Formats use these types rather than their own tuples or structs.
- Every type stored as floats reads and writes through `xrf_chunk::ChunkReadWrite`, in the caller's byte order.
- All serialize with Serde in camelCase.

## Read, write and transform

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

## Vectors

`Vector3d<T = f32>` has three named components; the math is defined for `f32`.

- Constructors and constants: `new`, `ZERO`, `X`, `Y`, `Z`.
- Math: `dot`, `cross`, `length`, `distance` and their squared forms, `normalize_or_zero` (zero for a zero vector),
  component-wise `min` and `max`, `midpoint`, `to_array`.
- Operators on references: `&a + &b`, `&a - &b`, `&a * 2.0`, `&a / 2.0`, `-&a`, `sum += &a`.
- Converts from `[f32; 3]` and `(f32, f32, f32)`.
- Displays as `x,y,z` and parses that back with `FromStr` (whitespace around numbers allowed), failing with
  `XrfError::Parsing`.

`from_compressed_normal` and `to_compressed_normal` are `pvDecompress` and `pvCompress`, the 16-bit normal an AI node
stores its plane in, computed as SDK 0.7's xrCore computes them: the table at x87 extended precision, the rest in single
precision, the compression's sum `(z + y) + x`, a sign bit forced on rather than negated. All 65,536 values round trip
through decompression, normalisation and compression as SDK 0.7's xrAI saves them.

`Vector2d` is the same for two `f32` components: `new`, `ZERO`, `to_array`, and `x,y` through `Display` and `FromStr`.
It holds a texture coordinate or a sprite's size on its sheet.

## Matrices

`Matrix4x4` is the engine's full `Fmatrix`: sixteen `f32`s, row major, read and written as the engine copies it
(`ELEMENTS`, `SERIALIZED_SIZE`).

- The fourth row is the translation (`get_translation`, the engine's `Fmatrix::c`).
- `transform_point` applies the translation; `transform_direction` uses only the upper three rows, so directions and
  normals are not moved.
- `multiply` is `Fmatrix::mul`, the inner matrix applied first. Each element is summed in the engine's order, so a
  transform composed as the engine composes it has identical bits, signs of zeros included.
- `IDENTITY`, `from_translation` (`translate`), `from_scale` (`scale`) and `From<&Affine3d>` build matrices with a
  fourth column of `0, 0, 0, 1`. `to_affine` and `get_fourth_column` split one back.

`Affine3d` is the same matrix as `NET_Packet::w_matrix` writes it: rows `i`, `j`, `k`, `c` as `x_axis`, `y_axis`,
`z_axis` and `translation`, with the fourth column fixed at `0, 0, 0, 1` and not stored. `transform_point` is
`Fmatrix::transform_tiny`. A spawn shape's box is one.

Where the engine reads `sizeof(Fmatrix)`, use `Matrix4x4`. `Affine3d` reads only 48 of the 64 bytes and may still
parse, misaligning every later read.

## Colour and rotation

- `Color` is the engine's `Fcolor`: four `f32`s stored red, green, blue, alpha. A level's static lights carry three.
  It is not a packed four-byte `D3DCOLOR`.
- `Quaternion` is the engine's `Fquaternion`: `x`, `y`, `z`, then `w`, with `IDENTITY` for no rotation. Skeleton motions
  store rotations quantised, so `xrf-skeleton` decodes each key into one.

## Shapes

Each lays out as the engine's own type, field by field in the engine's order (`SERIALIZED_SIZE` on each):

| Type       | Engine      | Fields                                                                 | Bytes |
| ---------- | ----------- | ---------------------------------------------------------------------- | ----- |
| `Aabb`     | `Fbox`      | `min`, `max`                                                           | 24    |
| `Obb`      | `Fobb`      | `rotation` (`m_rotate`, rows are its axes), `center`, `half_extents`   | 60    |
| `Sphere`   | `Fsphere`   | `center` (`P`), `radius` (`R`)                                         | 16    |
| `Cylinder` | `Fcylinder` | `center`, `direction`, `height` (full height along the axis), `radius` | 32    |

Where they appear:

- An OGF header bounds its visual by an `Aabb` and a `Sphere`; each bone name carries an `Obb`.
- A bone's collision shape (`xrf-skeleton`'s `SkeletonBoneShape`) holds an `Obb`, a `Sphere` and a `Cylinder`.
- Editable meshes, a level's collision form, AI map and sound environment, and `build.cform` and `build.aimap` are
  bounded by an `Aabb`. A spawn shape's sphere is a `Sphere`.

A shape carries no coordinate space. `xrf-visual`'s `VisualBounds`, for example, holds an `Aabb` and a `Sphere` in
renderer space and says so.

`Aabb` operations:

- `from_points` is the box around a run of points; `None` for no points (the engine uses an inverted box).
- To grow one point by point, start with `of_point` and add each with `include_point` (`Fbox::modify`). Ties keep the
  existing side, as `std::min` and `std::max` do, so a zero of the other sign keeps its bits.
- `include_box` adds a whole box (`Fbox::merge`).
- `transform` is `Fbox::xform`: the box around one placed by a matrix, summed in the engine's order.
- `get_size`, `get_center` and `get_bounding_sphere` are `Fbox::getsize`, `getcenter` and `getsphere`.

## AI node positions

`AiNodePosition` is where the AI map tool's part and `build.aimap` keep a node: its cell along x and z in patches from
the world origin, and its height packed across a box's height. `pack` and `unpack` are the editor's `PackPosition` and
`UnpackPosition`; `pack_height` and `unpack_height` are the height rule `level.ai` shares, over its own extent.

## Epsilons

`EPS_S` (`0.0000001`), `EPS` (`0.00001`) and `EPS_L` (`0.001`) are the engine's epsilons from
`xrCore/math_constants.h`, under the engine's names. `EPS_S` is what `fis_zero` compares against by default.

- Never redefine one locally. Give it a name that says what it means there: `const GROWTH: f32 = EPS_L`.
- A value the engine writes as a literal stays a literal.
- The UI has the same constants in `@xrf/math`.

## Display colour

- `srgb_decoded` and `srgb_encoded` convert a display colour to linear light and back, with the sRGB reference
  display's 2.2 power (`SRGB_DISPLAY_POWER`).
- `luminance` weights linear light by Rec. 709 (`LUMINANCE_WEIGHTS`).

The renderer lights with these (its `lighting/srgb.rs` and `lighting/luminance.rs` mirror `common/srgb.wgsl` and
`common/luminance.wgsl` through them), and `xrf-texture`'s albedo measure reads textures with them, so the measure
agrees with what the renderer draws.

## Features

- `typescript-bindings` derives Specta types for every type except `Matrix4x4`.
- `fixtures` exports `Vector3d::new_mock()`, the vector this crate's tests use, for dependants' tests.
- `glam` converts `Vector3d` and `Vector2d` to and from `glam`'s `Vec3` and `Vec2` with `From`, for runtime crates that
  compute with `glam` (particles, world, renderer, the levels plugin). The copy is component-wise and changes no space,
  so any change of space stays visible where it is written.

## Checks

Run `cargo test --locked -p xrf-math` from the repository root.

Sources: [Vector2d](src/vector_2d.rs), [Vector3d](src/vector_3d.rs), [Matrix4x4](src/matrix_4x4.rs),
[Affine3d](src/affine_3d.rs), [Quaternion](src/quaternion.rs), [Color](src/color.rs), the [shapes](src/shape/),
[AiNodePosition](src/ai_node_position.rs), the
[glam conversions](src/glam_conversions.rs) and the [epsilons](src/math_constants.rs).
