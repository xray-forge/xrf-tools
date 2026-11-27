use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_game_loop::{GameClock, LoopFeed, LoopStepper, LoopTime};
use xrf_game_movement::{MovementCamera, MovementCommand};
use xrf_math::glam::Vec3;
use xrf_world::{WorldFedCamera, WorldFedView, WorldViewFeed};

use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;

fn assert_near(actual: Vec3, expected: Vec3) {
  assert!(actual.abs_diff_eq(expected, 1e-4), "{actual} is not {expected}");
}

#[test]
fn routes_a_command_to_its_system_and_publishes_the_camera() {
  let mut stepper: LoopStepper<GameState> =
    LoopStepper::new(Duration::from_millis(10), GameClock::default(), GameState::default());
  let camera: MovementCamera = MovementCamera::looking_at(Vec3::new(1.0, 2.0, 3.0), Vec3::new(1.0, 2.0, 9.0));

  stepper.step([GameCommand::Movement(MovementCommand::SetCamera(camera))]);

  assert_eq!(stepper.get_frame().state.camera, camera);
  assert_eq!(*stepper.get_state().movement.get_camera(), camera);
}

#[test]
fn feeds_the_world_the_loops_camera_through_its_lens_and_the_games_day() {
  let feed: Arc<LoopFeed<GameFrame>> = Arc::new(LoopFeed::default());
  let lens: WorldFedCamera = WorldFedCamera {
    field_of_view: 75.0,
    ..WorldFedCamera::default()
  };
  let view: GameViewFeed = GameViewFeed::new(Arc::clone(&feed), lens);
  let now: Instant = Instant::now();

  // Before the loop's first frame, the lens alone and the viewport's own clock.
  assert_eq!(
    view.read(now),
    WorldFedView {
      camera: lens,
      clock: None
    }
  );

  let mut stepper: LoopStepper<GameState> =
    LoopStepper::new(Duration::from_millis(10), GameClock::default(), GameState::default());

  stepper.step([GameCommand::Movement(MovementCommand::SetCamera(
    MovementCamera::looking_at(Vec3::new(4.0, 5.0, 6.0), Vec3::new(5.0, 5.0, 6.0)),
  ))]);
  feed.publish(now, stepper.get_frame());

  let read: WorldFedView = view.read(now);
  let time: LoopTime = *stepper.get_time();

  assert_near(read.camera.position, Vec3::new(4.0, 5.0, 6.0));
  assert_near(read.camera.forward, Vec3::X);
  assert_near(read.camera.up, Vec3::Y);
  assert_eq!(read.camera.field_of_view, 75.0);

  let clock = read.clock.expect("the game runs the day");

  assert!((clock.time - time.get_time_of_day()).abs() < 1e-3);
  assert_eq!(clock.rate, 10.0);
}
