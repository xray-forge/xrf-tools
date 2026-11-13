use xrf_math::{EPS, Vector3d};

use crate::triangle::face_area::FaceArea;

#[test]
fn measures_a_right_triangle_by_heron() {
  let corners: [Vector3d; 3] = [
    Vector3d::ZERO,
    Vector3d::new(2.0, 0.0, 0.0),
    Vector3d::new(0.0, 0.0, 3.0),
  ];

  assert!((FaceArea::of(&corners) - 3.0).abs() < 1e-5);
  assert!(FaceArea::is_enough(&corners));
}

#[test]
fn refuses_a_face_under_the_least_area_and_one_with_no_area() {
  // A sliver whose area is half of `EPS`.
  let sliver: [Vector3d; 3] = [
    Vector3d::ZERO,
    Vector3d::new(1.0, 0.0, 0.0),
    Vector3d::new(0.0, 0.0, EPS),
  ];
  let point: [Vector3d; 3] = [Vector3d::ZERO; 3];

  assert!(!FaceArea::is_enough(&sliver));
  assert!(!FaceArea::is_enough(&point));
}
