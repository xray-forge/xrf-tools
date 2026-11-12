use glam::Vec3;

use crate::ray::facing::Facing;
use crate::ray::ray::Ray;
use crate::ray::ray_crossing::RayCrossing;
use crate::triangle::triangle::Triangle;

/// A right triangle on the ground, its front facing up: `(p1 - p0) x (p2 - p0)` is `+y`.
fn ground() -> Triangle {
  Triangle::new([Vec3::ZERO, Vec3::new(0.0, 0.0, 2.0), Vec3::new(2.0, 0.0, 0.0)])
}

#[test]
fn crosses_a_front_face_with_its_distance_and_barycentrics() {
  let crossing: RayCrossing = ground()
    .cross(&Ray::new(Vec3::new(0.5, 3.0, 1.0), Vec3::NEG_Y, 10.0), Facing::Front)
    .expect("a crossing");

  assert_eq!(
    crossing,
    RayCrossing {
      distance: 3.0,
      u: 0.5,
      v: 0.25,
    }
  );
}

#[test]
fn crosses_a_back_face_only_from_both_sides() {
  let ray: Ray = Ray::new(Vec3::new(0.5, -3.0, 1.0), Vec3::Y, 10.0);

  assert_eq!(ground().cross(&ray, Facing::Front), None);
  assert_eq!(
    ground().cross(&ray, Facing::Both).map(|crossing| crossing.distance),
    Some(3.0)
  );
}

#[test]
fn keeps_a_crossing_ahead_of_the_origin_up_to_the_range_inclusive() {
  let from =
    |height: f32, range: f32| ground().cross(&Ray::new(Vec3::new(0.5, height, 1.0), Vec3::NEG_Y, range), Facing::Both);

  assert!(from(3.0, 3.0).is_some());
  assert!(from(3.0, 2.999).is_none());
  assert!(from(0.0, 1.0).is_none(), "a crossing at the origin is not ahead of it");
  assert!(from(-1.0, 5.0).is_none(), "nor one behind it");
}

#[test]
fn crosses_an_edge_and_a_corner_and_misses_beside_them() {
  let down = |x: f32, z: f32| ground().cross(&Ray::new(Vec3::new(x, 1.0, z), Vec3::NEG_Y, 2.0), Facing::Front);

  assert!(down(0.0, 1.0).is_some(), "on an edge");
  assert!(down(0.0, 0.0).is_some(), "on a corner");
  assert!(down(1.0, 1.0).is_some(), "on the long edge");
  assert!(down(1.01, 1.01).is_none());
  assert!(down(-0.01, 1.0).is_none());
}

// `CDB::TestRayTri` takes a determinant under `EPS` as no crossing: a ray along the triangle's plane, and a triangle a
// couple of millimetres across.
#[test]
fn crosses_neither_along_the_plane_nor_a_triangle_too_small_to_tell() {
  let tiny: Triangle = Triangle::new([Vec3::ZERO, Vec3::new(0.0, 0.0, 0.002), Vec3::new(0.002, 0.0, 0.0)]);

  assert_eq!(
    ground().cross(&Ray::new(Vec3::new(-1.0, 0.0, 0.5), Vec3::X, 10.0), Facing::Both),
    None
  );
  assert_eq!(
    tiny.cross(
      &Ray::new(Vec3::new(0.0005, 1.0, 0.0005), Vec3::NEG_Y, 2.0),
      Facing::Both
    ),
    None
  );
}

#[test]
fn overlaps_a_box_it_passes_through_or_touches() {
  assert!(ground().overlaps_box(Vec3::new(0.5, 0.0, 0.5), Vec3::splat(0.1)));
  assert!(
    ground().overlaps_box(Vec3::new(0.5, 0.5, 0.5), Vec3::splat(0.5)),
    "touching from above"
  );
  assert!(!ground().overlaps_box(Vec3::new(0.5, 0.51, 0.5), Vec3::splat(0.5)));
  assert!(!ground().overlaps_box(Vec3::new(5.0, 0.0, 5.0), Vec3::splat(1.0)));
}

// The box's corner lies beyond the triangle's long edge: neither the box's axes nor the triangle's plane separate
// them, only an edge's cross product with an axis does.
#[test]
fn overlaps_no_box_past_its_long_edge() {
  let corner_off: Vec3 = Vec3::new(1.3, 0.0, 1.3);

  assert!(!ground().overlaps_box(corner_off + Vec3::new(0.2, 0.0, 0.2), Vec3::splat(0.2)));
  assert!(ground().overlaps_box(corner_off - Vec3::new(0.25, 0.0, 0.25), Vec3::splat(0.2)));
}

#[test]
fn faces_its_normal_the_way_its_front_does() {
  assert_eq!(ground().get_normal(), Vec3::Y);
  assert_eq!(Triangle::new([Vec3::ZERO; 3]).get_normal(), Vec3::ZERO);
}

/// `CDB::TestRayTri` in plain three-component arithmetic, each operation as the engine writes it.
fn cross_scalar(corners: [Vec3; 3], origin: Vec3, direction: Vec3, facing: Facing) -> Option<[f32; 3]> {
  let edge1: Vec3 = corners[1] - corners[0];
  let edge2: Vec3 = corners[2] - corners[0];
  let pvec: Vec3 = direction.cross(edge2);
  let det: f32 = edge1.dot(pvec);
  let tvec: Vec3 = origin - corners[0];
  let qvec: Vec3 = tvec.cross(edge1);

  match facing {
    Facing::Front => {
      let (u, v) = (tvec.dot(pvec), direction.dot(qvec));

      (det >= xrf_math::EPS && (0.0..=det).contains(&u) && v >= 0.0 && u + v <= det).then(|| {
        let inverse: f32 = 1.0 / det;

        [edge2.dot(qvec) * inverse, u * inverse, v * inverse]
      })
    }
    Facing::Both => {
      let inverse: f32 = 1.0 / det;
      let (u, v) = (tvec.dot(pvec) * inverse, direction.dot(qvec) * inverse);

      (det.abs() >= xrf_math::EPS && (0.0..=1.0).contains(&u) && v >= 0.0 && u + v <= 1.0)
        .then(|| [edge2.dot(qvec) * inverse, u, v])
    }
  }
}

// The corners and rays are held a SIMD register wide; every crossing must still come out bit for bit as the scalar
// arithmetic gives it, distance and barycentrics alike, or the light that rests on them would move.
#[test]
fn crosses_bit_for_bit_as_the_scalar_arithmetic_does() {
  let mut state: u64 = 3;
  let mut next = || {
    state = state
      .wrapping_mul(6_364_136_223_846_793_005)
      .wrapping_add(1_442_695_040_888_963_407);

    (state >> 40) as f32 / (1u64 << 24) as f32
  };
  let mut crossed: usize = 0;

  for _ in 0..20_000 {
    let corners: [Vec3; 3] = std::array::from_fn(|_| Vec3::new(next(), next(), next()) * 10.0);
    let target: Vec3 = corners[0] + (corners[1] - corners[0]) * next() * 0.6 + (corners[2] - corners[0]) * next() * 0.6;
    let origin: Vec3 = Vec3::new(next(), next(), next()) * 30.0 - Vec3::splat(10.0);
    let direction: Vec3 = (target - origin).normalize_or(Vec3::Y);

    for facing in [Facing::Front, Facing::Both] {
      let found: Option<[f32; 3]> = Triangle::new(corners)
        .cross(&Ray::new(origin, direction, f32::MAX), facing)
        .map(|crossing| [crossing.distance, crossing.u, crossing.v]);
      let expected: Option<[f32; 3]> =
        cross_scalar(corners, origin, direction, facing).filter(|[distance, ..]| *distance > 0.0);

      crossed += usize::from(found.is_some());
      assert_eq!(
        found.map(|values| values.map(f32::to_bits)),
        expected.map(|values| values.map(f32::to_bits)),
        "{corners:?} from {origin} along {direction}, {facing:?}"
      );
    }
  }

  assert!(crossed > 10_000, "the rays aim at the triangles: {crossed}");
}
