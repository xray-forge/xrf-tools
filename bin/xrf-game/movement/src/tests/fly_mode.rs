use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use xrf_math::glam::{Vec2, Vec3};

use crate::camera::movement_camera::MovementCamera;
use crate::camera::movement_heading::MovementHeading;
use crate::input::movement_action::MovementAction;
use crate::input::movement_input_kind::MovementInputKind;
use crate::input::movement_input_state::MovementInputState;
use crate::mode::fly_mode::FlyMode;

const SECOND: Duration = Duration::from_secs(1);

fn assert_near(actual: Vec3, expected: Vec3) {
  assert!(actual.abs_diff_eq(expected, 1e-4), "{actual} is not {expected}");
}

fn holding(actions: &[MovementAction]) -> MovementInputState {
  let mut input: MovementInputState = MovementInputState::default();

  for action in actions {
    input.apply(MovementInputKind::Action {
      action: *action,
      is_pressed: true,
    });
  }

  input
}

fn fly(camera: &mut MovementCamera, input: &mut MovementInputState, step: Duration) {
  FlyMode::default().step(camera, input, step);
}

#[test]
fn flies_where_it_looks_at_its_speed_and_faster_with_the_boost() {
  let mut camera: MovementCamera = MovementCamera::looking_at(Vec3::ZERO, Vec3::new(1.0, 1.0, 0.0));
  let mut input: MovementInputState = holding(&[MovementAction::Forward]);

  fly(&mut camera, &mut input, SECOND);
  assert_near(camera.position, Vec3::new(1.0, 1.0, 0.0).normalize() * 10.0);

  let mut camera: MovementCamera = MovementCamera::default();
  let mut input: MovementInputState = holding(&[MovementAction::Back, MovementAction::Accel]);

  fly(&mut camera, &mut input, Duration::from_millis(500));
  assert_near(camera.position, Vec3::new(0.0, 0.0, -20.0));
}

#[test]
fn strafes_along_its_right_and_rises_straight_up_however_it_is_pitched() {
  let mut camera: MovementCamera = MovementCamera::looking_at(Vec3::ZERO, Vec3::new(0.0, -1.0, 1.0));
  let mut input: MovementInputState = holding(&[MovementAction::Right]);

  fly(&mut camera, &mut input, SECOND);
  // Facing `+z`, its right is `+x`, as X-Ray's is.
  assert_near(camera.position, Vec3::new(10.0, 0.0, 0.0));

  let mut camera: MovementCamera = MovementCamera::looking_at(Vec3::ZERO, Vec3::new(0.0, -1.0, 1.0));
  let mut input: MovementInputState = holding(&[MovementAction::Up]);

  fly(&mut camera, &mut input, SECOND);
  assert_near(camera.position, Vec3::new(0.0, 10.0, 0.0));
}

#[test]
fn flies_a_diagonal_no_faster_than_a_straight_line() {
  let mut camera: MovementCamera = MovementCamera::default();
  let mut input: MovementInputState = holding(&[MovementAction::Forward, MovementAction::Right]);

  fly(&mut camera, &mut input, SECOND);
  assert!((camera.position.length() - 10.0).abs() < 1e-4);
}

#[test]
fn turns_right_and_looks_down_as_the_mouse_moves_right_and_down_once() {
  let mut camera: MovementCamera = MovementCamera::default();
  let mut input: MovementInputState = MovementInputState::default();
  let quarter: f32 = FRAC_PI_2 / FlyMode::default().sensitivity;

  input.apply(MovementInputKind::Look { dx: quarter, dy: 0.0 });
  fly(&mut camera, &mut input, SECOND);
  assert_near(camera.get_forward(), Vec3::X);

  // The counts were taken: the next step turns no further.
  fly(&mut camera, &mut input, SECOND);
  assert_near(camera.get_forward(), Vec3::X);
  assert_eq!(input.take_look(), Vec2::ZERO);

  input.apply(MovementInputKind::Look {
    dx: 0.0,
    dy: quarter * 3.0,
  });
  fly(&mut camera, &mut input, SECOND);
  // Held just short of straight down.
  assert!((camera.get_heading().pitch - MovementHeading::MOST_PITCH).abs() < 1e-4);
  assert!(camera.get_forward().y < -0.99);
}
