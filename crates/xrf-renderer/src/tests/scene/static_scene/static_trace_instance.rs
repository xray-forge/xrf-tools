//! What a top-level instance's transform does to its cut-out triangles' densities.

use glam::{Mat4, Quat, Vec3};

use crate::scene::static_scene::trace::static_trace_group::StaticTraceGroup;
use crate::scene::static_scene::trace::static_trace_instance::StaticTraceInstance;

fn create_instance(matrix: Mat4) -> StaticTraceInstance {
  StaticTraceInstance::new(3, 5, &StaticTraceGroup::pack_transform(&matrix))
}

/// An instance standing its geometry as it is, or only moved, leaves its densities as they are.
#[test]
fn leaves_the_density_of_an_instance_only_moved() {
  let instance: StaticTraceInstance = create_instance(Mat4::from_translation(Vec3::new(10.0, -2.0, 7.0)));

  assert_eq!((instance.first_record, instance.place, instance.flags), (3, 5, 0));
  assert!(instance.density_scale.abs() < 1e-6, "{instance:?}");
}

/// An instance turned and doubled halves its triangles' densities, a turn and a scale alike each way, whatever way they
/// face: one level of detail less.
#[test]
fn takes_a_level_off_an_instance_turned_and_doubled() {
  let instance: StaticTraceInstance = create_instance(Mat4::from_scale_rotation_translation(
    Vec3::splat(2.0),
    Quat::from_euler(glam::EulerRot::YXZ, 0.7, -0.3, 1.1),
    Vec3::new(4.0, 0.0, -9.0),
  ));

  assert_eq!(instance.flags, 0);
  assert!((instance.density_scale + 1.0).abs() < 1e-5, "{instance:?}");
}

/// An instance stretched along one axis is marked, so each triangle's density is worked out by the way it faces where
/// it is read.
#[test]
fn marks_an_instance_scaled_unevenly() {
  let instance: StaticTraceInstance = create_instance(Mat4::from_scale(Vec3::new(1.0, 1.5, 1.0)));

  assert_eq!(instance.flags, StaticTraceInstance::UNEVEN);
  assert_eq!(instance.density_scale, 0.0);
}
