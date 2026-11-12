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
