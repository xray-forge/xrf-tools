# xrf-geometry

Geometry the level tools compute over: ray and box queries against a set of triangles, in the arithmetic of the
engine's collision database (`xrCDB`). It reads no file format; callers hand it vertices and triangles and look up what
a face is made of themselves. The level viewer, the decompiler and the compilers use it alike.

| Type        | What it is                                                                                        |
| ----------- | ------------------------------------------------------------------------------------------------- |
| `Tracer`    | The queries: every hit along a ray (one ray or a packet), faces in a box, nearest, blocked, along |
| `BvhTracer` | The tracer: `obvhs`' PLOC binary hierarchy walked by its own exact traversal                      |
| `Triangle`  | `CDB::TestRayTri` (`cross`), the separating axis test against a box, `Fvector::mknormal`          |
| `Ray`       | Origin, direction and range; a crossing counts ahead of the origin up to the range, inclusive     |
| `TraceHit`  | A face a ray meets: its index, distance and barycentrics                                          |
| `Facing`    | Whether a ray test meets a triangle's front only (`OPT_CULL`) or either side                      |

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
test pins, and the queries run about 1.15 times faster for it. Its box test takes a zero direction component's inverse as infinite
and an origin exactly on a box's face as inside, where `obvhs`' own traversals lose faces on such rays.
`for_each_packet_hit` traces many rays together, 256 at a time, which pays where they run close together, such as one
set of points' rays toward one light.

## Develop

```bash
cargo test -p xrf-geometry
```
