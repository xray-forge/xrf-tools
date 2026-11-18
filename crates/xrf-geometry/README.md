# xrf-geometry

Geometry the level tools compute over: ray and box queries against a set of triangles, in the arithmetic of the
engine's collision database (`xrCDB`), and the mesh work the SDK's exports and xrLC share. It reads no file format;
callers hand it vertices and triangles and look up what a face is made of themselves. The level viewer, the decompiler,
the object compile and the level compilers use it alike.

| Type                 | What it is                                                                                        |
| -------------------- | ------------------------------------------------------------------------------------------------- |
| `Tracer`             | The queries: every hit along a ray (one ray or a packet), faces in a box, nearest, blocked, along |
| `BvhTracer`          | The tracer: `obvhs`' PLOC binary hierarchy walked by its own exact traversal                      |
| `XrlcTracer`         | xrLC's ray model: xrCDB's Opcode tree and ray walk, which loses the rays xrLC loses at box corners |
| `Triangle`           | `CDB::TestRayTri` (`cross`), the separating axis test against a box, `Fvector::mknormal`          |
| `Ray`                | Origin, direction and range; a crossing counts ahead of the origin up to the range, inclusive     |
| `TraceHit`           | A face a ray meets: its index, distance and barycentrics                                          |
| `Facing`             | Whether a ray test meets a triangle's front only (`OPT_CULL`) or either side                      |
| `Box2Pick`           | `Fbox2::pick_exact`: whether a line across the ground meets a box, as xrAI's straight walks test  |
| `MeshVertex`, `Mesh` | A split's vertex (position, basis, coordinate, bones) and its faces, 32-bit packed or 16-bit cut  |
| `VertexPacker`       | Corners packed into vertices over a grid of the object's box, as the exports' `VPack`             |
| `TangentBasis`       | The exports' MeshMender tangent pass: splits, orders and shifts a split's vertices                |
| `SplitPartition`     | A split cut into children 16-bit indices reach                                                    |
| `VertexCacheOrder`   | A child's faces ordered for the vertex cache (Forsyth), its vertices by first use                 |
| `PackGrid`           | The grid `VPack` files packed vertices in, shared by the exports' and xrLC's packers              |
| `PointWelder`        | Points welded within `EPS_L`, as the editor's `VCPacked::add_vert`                                |
| `ProgressiveMesh`    | A mesh made progressive by sliding windows, as ETools' `VIPM_Convert` (Forsyth's VIPM, a quadric) |
| `ProgressivePoint`   | A vertex of a mesh made progressive: the position and coordinate a collapse is measured by        |
| `ProgressiveWindow`  | One level of detail: its run of the index buffer and the vertices it reaches                      |
| `FaceArea`           | `CalcArea`, Heron's formula in single precision, and the `EPS` a face must reach                  |
| `Sky`, `SkyLight`    | The editor's generated sky (`xrHemisphereBuild`) and sun by quality, each light a `SkyLight`      |

```rust
use glam::Vec3;
use xrf_geometry::{BvhTracer, Ray, Tracer};

let tracer: BvhTracer = BvhTracer::new(
  &[Vec3::new(-5.0, 0.0, -5.0), Vec3::new(5.0, 0.0, -5.0), Vec3::new(5.0, 0.0, 5.0), Vec3::new(-5.0, 0.0, 5.0)],
  &[[0, 2, 1], [0, 3, 2]],
);
let down: Ray = Ray::new(Vec3::new(1.0, 2.0, 1.0), Vec3::NEG_Y, 5.0);

assert!(tracer.is_blocked(&down));
assert_eq!(tracer.get_nearest_hit(&down).map(|hit| hit.distance), Some(2.0));
```

Every ray test is `Triangle::cross`, so a query answers as testing every face in turn would, whichever implementation
runs it; the tests hold `BvhTracer` to such a walk. A triangle's corners and a ray's vectors are `glam`'s `Vec3A`, a
SIMD register wide, built from `Vec3`: each crossing comes out bit for bit as the scalar arithmetic gives it, which a
test pins, and the queries run about 1.15 times faster for it. Its box test takes a zero direction component's inverse
as infinite and an origin exactly on a box's face as inside, where `obvhs`' own traversals lose faces on such rays.
`for_each_packet_hit` traces many rays together, 256 at a time, which pays where they run close together, such as one
set of points' rays toward one light.

`XrlcTracer` is for results matched point for point against xrLC's. It builds the tree xrCDB builds with Opcode
(`MODEL::build_internal`, `xrCDB/xrCDB.cpp`) and walks it as `ray_collider::_stab` does (`xrCDB/xrCDB_ray.cpp`), its
box test `isect_sse` lane for lane. Opcode keeps a node's box as a centre and half extents, and given back they can end
a rounding short of a corner: a ray along an axis exactly there misses the node and every face below it, in xrLC and
here alike, where `BvhTracer` meets them. Its tests hold it to a walk over every face elsewhere.

## Develop

```bash
cargo test -p xrf-geometry
```
