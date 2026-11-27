use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use xrf_game_loop::{LoopSystem, LoopTime};
use xrf_game_objects::ObjectRecord;
use xrf_math::glam::Vec3;
use xrf_physics::PhysicsWorld;

use crate::camera::movement_camera::MovementCamera;
use crate::camera::movement_heading::MovementHeading;
use crate::event::movement_event::MovementEvent;
use crate::input::movement_action::MovementAction;
use crate::input::movement_input::MovementInput;
use crate::input::movement_input_kind::MovementInputKind;
use crate::mode::movement_mode::MovementMode;
use crate::mode::movement_modes::MovementModes;
use crate::movement::Movement;
use crate::movement_command::MovementCommand;
use crate::movement_context::MovementContext;
use crate::tests::fixtures::player;

fn input(milliseconds: u64, kind: MovementInputKind) -> MovementCommand {
  MovementCommand::Input(MovementInput {
    at: Duration::from_millis(milliseconds),
    kind,
  })
}

fn press(milliseconds: u64, action: MovementAction, is_pressed: bool) -> MovementCommand {
  input(milliseconds, MovementInputKind::Action { action, is_pressed })
}

#[test]
fn holds_what_the_player_pressed_until_released() {
  let mut movement: Movement = Movement::default();

  movement.apply(press(1, MovementAction::Forward, true));
  movement.apply(press(2, MovementAction::Accel, true));

  assert_eq!(
    movement
      .get_input()
      .get_axis(MovementAction::Forward, MovementAction::Back),
    1.0
  );
  assert!(movement.get_input().is_held(MovementAction::Accel));

  movement.apply(press(3, MovementAction::Back, true));
  assert_eq!(
    movement
      .get_input()
      .get_axis(MovementAction::Forward, MovementAction::Back),
    0.0
  );

  movement.apply(input(4, MovementInputKind::ReleaseAll));
  assert!(!movement.get_input().is_held(MovementAction::Forward));
  assert!(!movement.get_input().is_held(MovementAction::Accel));
}

/// Steps `movement` once over `physics`, writing `player` and answering the events it told.
fn step_once(movement: &mut Movement, physics: &mut PhysicsWorld, player: &mut ObjectRecord) -> Vec<MovementEvent> {
  let mut events: Vec<MovementEvent> = Vec::new();

  movement
    .step(
      &LoopTime::new(Duration::from_millis(10)),
      MovementContext {
        physics,
        player,
        events: &mut events,
      },
    )
    .expect("a step");

  events
}

#[test]
fn faces_where_presentation_turned_it_and_flies_along_it() {
  let mut movement: Movement = Movement::default();
  let mut physics: PhysicsWorld = PhysicsWorld::new().expect("a world");
  let mut player: ObjectRecord = player();
  let heading: MovementHeading = MovementHeading {
    yaw: FRAC_PI_2,
    pitch: 0.0,
  };

  movement.apply(MovementCommand::Face(heading));
  movement.apply(input(
    0,
    MovementInputKind::Action {
      action: MovementAction::Forward,
      is_pressed: true,
    },
  ));
  step_once(&mut movement, &mut physics, &mut player);

  // A quarter turn faces `+x`, and the step flew 10 cm along it at 10 m/s.
  assert!(movement.get_camera().get_forward().abs_diff_eq(Vec3::X, 1e-5));
  assert!(
    movement
      .get_camera()
      .position
      .abs_diff_eq(Vec3::new(0.1, 0.0, 0.0), 1e-5)
  );
  assert!(movement.get_velocity().abs_diff_eq(Vec3::new(10.0, 0.0, 0.0), 1e-3));
  assert_eq!(movement.get_heading(), heading);
}

#[test]
fn stands_the_camera_where_a_command_puts_it_and_keeps_it_through_a_step() {
  let mut movement: Movement = Movement::default();
  let camera: MovementCamera = MovementCamera::looking_at(Vec3::new(5.0, 1.0, 2.0), Vec3::ZERO);
  let mut player: ObjectRecord = player();

  movement.apply(MovementCommand::SetCamera(camera));
  step_once(&mut movement, &mut PhysicsWorld::new().expect("a world"), &mut player);

  assert!(movement.get_camera().position.abs_diff_eq(camera.position, 1e-5));
  assert!(
    movement
      .get_camera()
      .get_forward()
      .abs_diff_eq(camera.get_forward(), 1e-5)
  );
  // A camera stood somewhere did not move there: no velocity to carry it on by.
  assert_eq!(movement.get_velocity(), Vec3::ZERO);
  // In flight the player is where the camera is.
  assert!(player.position.abs_diff_eq(camera.position, 1e-5));
}

#[test]
fn enters_another_level_at_its_start_leaving_the_walking_actor_behind() {
  let mut physics: PhysicsWorld = PhysicsWorld::new().expect("a world");
  let mut player: ObjectRecord = player();
  let mut movement: Movement = Movement::new(
    MovementCamera::looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.0, 2.0, 1.0)),
    MovementMode::Walk,
    MovementModes::default(),
  );
  let start: MovementCamera = MovementCamera::looking_at(Vec3::new(50.0, 3.0, 9.0), Vec3::new(51.0, 3.0, 9.0));

  step_once(&mut movement, &mut physics, &mut player);
  assert!(movement.get_walk().actor.is_some());

  movement.enter_level(start);

  assert_eq!(*movement.get_camera(), start);
  assert!(movement.get_walk().actor.is_none());
  assert_eq!(movement.get_mode(), MovementMode::Walk);
}
