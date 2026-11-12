use glam::Vec3A;

use crate::ray::slab_ray::SlabRay;

/// Whether a ray reaches the box a metre across from the origin.
fn reaches_unit_box(origin: [f32; 3], direction: [f32; 3], reach: f32) -> bool {
  SlabRay::new(Vec3A::from(origin), Vec3A::from(direction)).reaches(reach, Vec3A::ZERO, Vec3A::ONE)
}

#[test]
fn reaches_a_box_along_its_axes_and_misses_one_beside_them() {
  assert!(reaches_unit_box([0.5, 0.5, -1.0], [0.0, 0.0, 1.0], 5.0));
  assert!(!reaches_unit_box([0.5, 0.5, -1.0], [0.0, 0.0, 1.0], 0.5));
  assert!(!reaches_unit_box([1.5, 0.5, -1.0], [0.0, 0.0, 1.0], 5.0));
  assert!(!reaches_unit_box([0.5, 0.5, -1.0], [0.0, 0.0, -1.0], 5.0));
}

// A direction with a zero component from an origin on a face's plane runs along the face: the NaN the products give
// there counts as inside.
#[test]
fn reaches_a_box_along_a_face_from_an_origin_on_its_plane() {
  assert!(reaches_unit_box([0.0, 0.5, -1.0], [0.0, 0.0, 1.0], 5.0));
  assert!(reaches_unit_box([1.0, 0.5, -1.0], [0.0, 0.0, 1.0], 5.0));
  assert!(
    reaches_unit_box([1.0, 0.5, -1.0], [-0.0, 0.0, 1.0], 5.0),
    "a minus zero as well"
  );
  assert!(
    reaches_unit_box([1.0, 1.0, -1.0], [0.0, 0.0, 1.0], 5.0),
    "along an edge"
  );
  assert!(!reaches_unit_box([1.0001, 0.5, -1.0], [0.0, 0.0, 1.0], 5.0));
}

#[test]
fn reaches_a_box_of_no_depth() {
  let ray: SlabRay = SlabRay::new(Vec3A::new(0.5, 2.0, 0.5), Vec3A::NEG_Y);

  assert!(ray.reaches(5.0, Vec3A::new(0.0, 1.0, 0.0), Vec3A::new(1.0, 1.0, 1.0)));
  assert!(
    ray.reaches(1.0, Vec3A::new(0.0, 1.0, 0.0), Vec3A::new(1.0, 1.0, 1.0)),
    "at the reach exactly"
  );
  assert!(!ray.reaches(0.99, Vec3A::new(0.0, 1.0, 0.0), Vec3A::new(1.0, 1.0, 1.0)));
}

#[test]
fn reaches_a_box_holding_its_origin() {
  assert!(reaches_unit_box([0.5, 0.5, 0.5], [0.3, -0.4, 0.866], 0.01));
}
