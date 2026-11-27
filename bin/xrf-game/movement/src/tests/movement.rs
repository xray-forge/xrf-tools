use std::time::Duration;

use xrf_game_loop::{GameClock, LoopSystem, LoopTime};
use xrf_math::glam::Vec3;

use crate::camera::movement_camera::MovementCamera;
use crate::input::movement_action::MovementAction;
use crate::input::movement_input::MovementInput;
use crate::input::movement_input_kind::MovementInputKind;
use crate::movement::Movement;
use crate::movement_command::MovementCommand;

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
  movement.apply(press(2, MovementAction::Fast, true));

  assert_eq!(
    movement
      .get_input()
      .get_axis(MovementAction::Forward, MovementAction::Back),
    1.0
  );
  assert!(movement.get_input().is_held(MovementAction::Fast));

  movement.apply(press(3, MovementAction::Back, true));
  assert_eq!(
    movement
      .get_input()
      .get_axis(MovementAction::Forward, MovementAction::Back),
    0.0
  );

  movement.apply(input(4, MovementInputKind::ReleaseAll));
  assert!(!movement.get_input().is_held(MovementAction::Forward));
  assert!(!movement.get_input().is_held(MovementAction::Fast));
}

#[test]
fn gathers_the_mouse_until_a_movement_mode_takes_it() {
  let mut movement: Movement = Movement::default();

  movement.apply(input(1, MovementInputKind::Look { dx: 3.0, dy: -1.0 }));
  movement.apply(input(2, MovementInputKind::Look { dx: 2.0, dy: 4.0 }));

  let mut state = *movement.get_input();

  assert_eq!(state.take_look().to_array(), [5.0, 3.0]);
  assert_eq!(state.take_look().to_array(), [0.0, 0.0]);
}

#[test]
fn stands_the_camera_where_a_command_puts_it_and_keeps_it_through_a_step() {
  let mut movement: Movement = Movement::default();
  let camera: MovementCamera = MovementCamera::looking_at(Vec3::new(5.0, 1.0, 2.0), Vec3::ZERO);

  movement.apply(MovementCommand::SetCamera(camera));
  movement.step(&LoopTime::new(Duration::from_millis(10), GameClock::default()));

  assert_eq!(*movement.get_camera(), camera);
}
