use std::f32::consts::FRAC_PI_2;

use xrf_math::Vector3d;

use crate::data::visual::skeleton::visual_transform::VisualTransform;

// A point above a visual's origin stays above its place whatever its heading; renderer space runs `z` the other way.
#[test]
fn stands_a_point_where_a_spawned_object_stands_its_visual() {
  let transform: VisualTransform =
    VisualTransform::of_spawn(&Vector3d::new(10.0, 2.0, 5.0), &Vector3d::new(0.0, FRAC_PI_2, 0.0));
  let point: Vector3d = transform.apply_to_point(&Vector3d::new(0.0, 1.0, 0.0));

  assert!((point.x - 10.0).abs() < 1e-5 && (point.y - 3.0).abs() < 1e-5 && (point.z + 5.0).abs() < 1e-5);
}

// Mirrored back, a spawned object stands where the engine stands it: at its own position, `z` as the spawn writes it.
#[test]
fn mirrors_a_spawned_transform_back_to_engine_space() {
  let transform: VisualTransform =
    VisualTransform::of_spawn(&Vector3d::new(10.0, 2.0, 5.0), &Vector3d::new(0.0, FRAC_PI_2, 0.0)).mirrored();
  let point: Vector3d = transform.apply_to_point(&Vector3d::new(0.0, 1.0, 0.0));
  let matrix: [f32; 16] = transform.to_matrix();

  assert!((point.x - 10.0).abs() < 1e-5 && (point.y - 3.0).abs() < 1e-5 && (point.z - 5.0).abs() < 1e-5);
  assert_eq!(&matrix[12..], &[10.0, 2.0, 5.0, 1.0]);
  assert_eq!(transform.mirrored().mirrored(), transform);
}
