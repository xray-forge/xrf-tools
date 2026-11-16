use xrf_math::{Matrix4x4, Vector3d};

use crate::geometry::model::geometry_placement::GeometryPlacement;
use crate::geometry::model::geometry_placement_sum::GeometryPlacementSum;

/// A placement of `jupiter`'s whose `y` row's sums part in the last bit by their order, with a model corner it places.
fn tilted() -> (Matrix4x4, Vector3d) {
  let mut matrix: Matrix4x4 = Matrix4x4::IDENTITY;

  matrix.values = [
    0.798_307_3,
    0.369_943_83,
    -0.189_333_13,
    0.0,
    -0.332_496_37,
    0.814_547_66,
    0.189_626_56,
    0.0,
    0.249_302_28,
    -0.098_252_98,
    0.859_182_54,
    0.0,
    -25.129_417,
    16.702_892,
    -329.815_7,
    1.0,
  ];

  (matrix, Vector3d::new(1.184_342_5, 0.741_2, -0.597_011_9))
}

#[test]
fn sums_each_axis_in_the_order_it_is_given() {
  let (matrix, point) = tilted();
  let ordered: Vector3d = GeometryPlacement::transform(&matrix, &point);
  let yz_first: Vector3d = GeometryPlacement::transform_by(&matrix, &point, &[GeometryPlacementSum::YzFirst; 3]);

  // SDK 0.7's xrLC placed this corner first in a ray model face, at 17.803432 m up; left to right gives a bit more.
  assert_eq!(ordered.y, 17.803_434);
  assert_eq!(yz_first.y, 17.803_432);
  assert_eq!(
    GeometryPlacementSum::XzFirst.sum(1.0, 1e-8, -1.0, 0.0),
    1e-8,
    "x and z cancel before y joins"
  );
  assert_eq!(GeometryPlacementSum::Ordered.sum(1.0, 1e-8, -1.0, 0.0), 0.0);
}
