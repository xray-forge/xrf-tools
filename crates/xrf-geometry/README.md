# xrf-geometry

Geometry the level tools compute over: ray and box queries against a set of triangles, in the arithmetic of the
engine's collision database (`xrCDB`), and the mesh work the SDK's exports and xrLC share. It reads no file format;
callers hand it vertices and triangles and look up what a face is made of themselves. The level viewer, the decompiler,
the object compile and the level compilers use it alike.

| Type                 | What it is                                                                                         |
| -------------------- | -------------------------------------------------------------------------------------------------- |
| `Tracer`             | The queries: every hit along a ray (one ray or a packet), faces in a box, nearest, blocked, along  |
| `BvhTracer`          | The tracer: `obvhs`' PLOC binary hierarchy walked by its own exact traversal                       |
| `XrlcTracer`         | xrLC's ray model: xrCDB's Opcode tree and ray walk, which loses the rays xrLC loses at box corners |
| `Triangle`           | `CDB::TestRayTri` (`cross`), the separating axis test against a box, `Fvector::mknormal`           |
| `Ray`                | Origin, direction and range; a crossing counts ahead of the origin up to the range, inclusive      |
| `TraceHit`           | A face a ray meets: its index, distance and barycentrics                                           |
| `Facing`             | Whether a ray test meets a triangle's front only (`OPT_CULL`) or either side                       |
| `Box2Pick`           | `Fbox2::pick_exact`: whether a line across the ground meets a box, as xrAI's straight walks test   |
| `MeshVertex`, `Mesh` | A split's vertex (position, basis, coordinate, bones) and its faces, 32-bit packed or 16-bit cut   |
| `VertexPacker`       | Corners packed into vertices over a grid of the object's box, as the exports' `VPack`              |
| `TangentBasis`       | The exports' MeshMender tangent pass: splits, orders and shifts a split's vertices                 |
| `SplitPartition`     | A split cut into children 16-bit indices reach                                                     |
| `VertexCacheOrder`   | A child's faces ordered for the vertex cache (Forsyth), its vertices by first use                  |
| `PackGrid`           | The grid `VPack` files packed vertices in, shared by the exports' and xrLC's packers               |
| `PointWelder`        | Points welded within `EPS_L`, as the editor's `VCPacked::add_vert`                                 |
| `ProgressiveMesh`    | A mesh made progressive by sliding windows, as ETools' `VIPM_Convert` (Forsyth's VIPM, a quadric)  |
| `ProgressivePoint`   | A vertex of a mesh made progressive: the position and coordinate a collapse is measured by         |
| `ProgressiveWindow`  | One level of detail: its run of the index buffer and the vertices it reaches                       |
| `FaceArea`           | `CalcArea`, Heron's formula in single precision, and the `EPS` a face must reach                   |
| `Sky`, `SkyLight`    | The editor's generated sky (`xrHemisphereBuild`) and sun by quality, each light a `SkyLight`       |
| `SymmetricEigen`     | A symmetric 3x3 matrix's axes by Jacobi rotations, double precision, as the SDK's fits take them   |
| `ConvexOutline`      | The convex hull of points in a plane, counter-clockwise, no collinear corner (monotone chain)      |

```rust
use xrf_math::glam::Vec3;
use xrf_geometry::{BvhTracer, Ray, Tracer};

let tracer: BvhTracer = BvhTracer::new(
  &[Vec3::new(-5.0, 0.0, -5.0), Vec3::new(5.0, 0.0, -5.0), Vec3::new(5.0, 0.0, 5.0), Vec3::new(-5.0, 0.0, 5.0)],
  &[[0, 2, 1], [0, 3, 2]],
);
let down: Ray = Ray::new(Vec3::new(1.0, 2.0, 1.0), Vec3::NEG_Y, 5.0);

assert!(tracer.is_blocked(&down));
assert_eq!(tracer.get_nearest_hit(&down).map(|hit| hit.distance), Some(2.0));
```

## Tracers

Every ray test is `Triangle::cross`. `BvhTracer` answers as testing every face in turn would; `XrlcTracer` does too,
except for the corner rays below. The tests hold both to such a walk.

- `BvhTracer` keeps corners and ray vectors as `glam`'s `Vec3A`, which gives each crossing bit for bit as scalar
  arithmetic does and runs the queries about 1.15 times faster. Its box test takes a zero direction component's inverse
  as infinite and an origin on a box's face as inside, where `obvhs`' own traversals lose faces.
- `for_each_packet_hit` traces up to `BvhTracer::PACKET_RAYS` (256) rays at a time, which pays where they run close
  together, such as a set of points' rays toward one light.
- `XrlcTracer` is for results matched point for point against xrLC's. It builds xrCDB's Opcode tree
  (`MODEL::build_internal`) and walks it as `ray_collider::_stab` does, its box test `isect_sse` lane for lane. Opcode
  stores a node's box as a centre and half extents, which can end a rounding short of a corner, so a ray along an axis
  exactly there misses every face below that node, as in xrLC. `BvhTracer` meets those faces.

## Checks

```bash
cargo test -p xrf-geometry
```
