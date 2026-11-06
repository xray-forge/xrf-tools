# xrf-math

The engine's basic math types as the format crates read and write them: `Vector2d` and `Vector3d` (the engine's
`Fvector2` and `Fvector`), `Matrix4x4` (its full `Fmatrix`) and `Affine3d` (an `Fmatrix` without its fourth column),
`Quaternion` (`Fquaternion`), `Color` (`Fcolor`), the shapes `Aabb`, `Obb`, `Sphere` and `Cylinder`, and the engine's
float epsilons. It sits at the bottom of the format crates, depending only on `xrf-chunk` and `xrf-error`; spawn, level,
skeleton, visual, OGF and particle crates build on it. A format holds these rather than a field pair, a tuple or a
struct of its own. Every type a file stores as floats reads and writes through `xrf_chunk::ChunkReadWrite` in the byte
order the caller names, and all serialize with Serde in camelCase.

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

`Vector2d` is the same for two components and `f32` alone: `new`, `ZERO`, `to_array`, `x,y` through `Display` and
`FromStr`, 8 bytes (`SERIALIZED_SIZE`). It holds a texture coordinate or a sprite's size on its sheet.

## Matrix4x4

`Matrix4x4` is the engine's full `Fmatrix`: sixteen `f32`s, row major, 64 bytes (`ELEMENTS`, `SERIALIZED_SIZE`), read
and written as the engine copies it in and out of a file. The fourth row is the translation (`get_translation`, the
engine's `Fmatrix::c`). `transform_point` applies the translation; `transform_direction` uses only the upper three rows,
so normals and other directions are not moved.

`Affine3d` is the same matrix as `NET_Packet::w_matrix` writes it: its rows `i`, `j`, `k` and `c` as `x_axis`, `y_axis`,
`z_axis` and `translation`, without the fourth column the reader fixes at `0, 0, 0, 1`, 48 bytes. `transform_point` is
`Fmatrix::transform_tiny`. A spawn shape's box is one. Where the engine reads `sizeof(Fmatrix)`, use `Matrix4x4`: the
narrower type reads 48 of the 64 bytes and can still parse, leaving every later read misaligned.

## Color

`Color` is the engine's `Fcolor`: four `f32`s stored red, green, blue, alpha, 16 bytes; a level's static lights carry
three. It is not a packed `D3DCOLOR`, which is four bytes in another order.

## Quaternion

`Quaternion` is the engine's `Fquaternion`, `x`, `y`, `z` and then `w`, with `IDENTITY` for no rotation. A skeleton
motion stores its rotations quantised, so `xrf-skeleton` decodes each key into one rather than reading floats.

## Shapes

Each is named for what it is and lays out as the engine's own type, read and written field by field in the engine's
order (`SERIALIZED_SIZE` on each):

| Type       | Engine      | Fields                                                                   | Bytes |
| ---------- | ----------- | ------------------------------------------------------------------------ | ----- |
| `Aabb`     | `Fbox`      | `min`, `max`                                                             | 24    |
| `Obb`      | `Fobb`      | `rotation` (`m_rotate`, rows are its axes), `center`, `half_extents`     | 60    |
| `Sphere`   | `Fsphere`   | `center` (`P`), `radius` (`R`)                                           | 16    |
| `Cylinder` | `Fcylinder` | `center`, `direction`, `height` (the full height along the axis), `radius` | 32  |

An OGF's header bounds its visual by an `Aabb` and a `Sphere`, its bone names carry an `Obb` each, and a bone's
collision shape (`xrf-skeleton`'s `SkeletonBoneShape`) holds one of each of the last three. An editable object's mesh,
a level's collision form, AI map and sound environment, and the compilers' `build.cform` and `build.aimap` are bounded
by an `Aabb`; a spawn shape's sphere is a `Sphere`. A shape carries no space: `xrf-visual`'s `VisualBounds` holds an
`Aabb` and a `Sphere` in renderer space, and says so, as a field of engine space would.

`Aabb::from_points` is the box holding a run of points, `None` for none, where the engine's inverted box would stand for
nothing; one grown point by point starts at `of_point` and takes each next through `include_point` (`Fbox::modify`).
`get_size`, `get_center` and `get_bounding_sphere` are `Fbox::getsize`, `getcenter` and `getsphere`.

## Epsilons

`EPS_S` (`0.0000001`), `EPS` (`0.00001`) and `EPS_L` (`0.001`) are the engine's epsilons from
`xrCore/math_constants.h`, under the engine's names; `EPS_S` is what `fis_zero` compares against by default. Never
redefine one locally: give it a local name that says what it means there (`const GROWTH: f32 = EPS_L`). A value the
engine writes as a literal stays a literal. The UI has the same constants in `@xrf/math`.

## Display colour

`srgb_decoded` and `srgb_encoded` turn a colour stored for the display into linear light and back, by the 2.2 power of
the sRGB reference display (`SRGB_DISPLAY_POWER`), and `luminance` weighs linear light by Rec. 709 (`LUMINANCE_WEIGHTS`).
The renderer lights with them (its `lighting/srgb.rs` and `lighting/luminance.rs` mirror `common/srgb.wgsl` and
`common/luminance.wgsl` through them), and `xrf-texture`'s albedo measure reads textures by them, so a measure always
agrees with what the renderer draws.

## Features and checks

`typescript-bindings` derives Specta types for every type here but `Matrix4x4`. `fixtures` exports
`Vector3d::new_mock()`, the vector this crate's tests use, for dependants' tests. `glam` converts `Vector3d` and
`Vector2d` to and from `glam`'s `Vec3` and `Vec2` with `From`, for the runtime crates that compute with `glam` over
stored values (particles, world, renderer, the levels plugin): a component-wise copy that changes no space, so a
conversion between spaces stays where it is written.

Run `cargo test --locked -p xrf-math` from the repository root. See [Vector2d](src/vector_2d.rs),
[Vector3d](src/vector_3d.rs), [Matrix4x4](src/matrix_4x4.rs), [Affine3d](src/affine_3d.rs),
[Quaternion](src/quaternion.rs), [Color](src/color.rs), the [shapes](src/shape/), the
[glam conversions](src/glam_conversions.rs) and the [epsilons](src/math_constants.rs).
