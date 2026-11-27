use xrf_game_loop::LoopBlend;
use xrf_math::glam::Vec3;

use crate::camera::movement_camera::MovementCamera;

fn assert_near(actual: Vec3, expected: Vec3) {
  assert!(actual.abs_diff_eq(expected, 1e-5), "{actual} is not {expected}");
}

#[test]
fn looks_along_positive_z_unturned_as_xray_does() {
  let camera: MovementCamera = MovementCamera::default();

  assert_near(camera.get_forward(), Vec3::Z);
  assert_near(camera.get_up(), Vec3::Y);
}

#[test]
fn looks_at_its_target_level_without_roll() {
  let position: Vec3 = Vec3::new(1.0, 2.0, 3.0);

  for target in [
    Vec3::new(4.0, 2.0, 3.0),
    Vec3::new(1.0, 5.0, 4.0),
    Vec3::new(-2.0, -1.0, -3.0),
  ] {
    let camera: MovementCamera = MovementCamera::looking_at(position, target);

    assert_near(camera.get_forward(), (target - position).normalize());
    // Never rolled: its right stays level.
    assert!(camera.get_forward().cross(camera.get_up()).y.abs() < 1e-5);
  }

  assert_near(MovementCamera::looking_at(position, position).get_forward(), Vec3::Z);
}

#[test]
fn blends_position_and_turn_between_two_cameras() {
  let from: MovementCamera = MovementCamera::looking_at(Vec3::ZERO, Vec3::Z);
  let to: MovementCamera = MovementCamera::looking_at(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0));
  let half: MovementCamera = from.blend(&to, 0.5);

  assert_near(half.position, Vec3::new(1.0, 0.0, 0.0));
  assert_near(half.get_forward(), Vec3::new(1.0, 0.0, 1.0).normalize());
}
