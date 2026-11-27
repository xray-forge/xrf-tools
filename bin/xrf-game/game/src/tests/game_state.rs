use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_game_alife::{Alife, AlifeData, AlifeSettings};
use xrf_game_loop::{GameClock, LoopFeed, LoopFrame, LoopStepper, LoopTime};
use xrf_game_movement::Movement;
use xrf_game_movement::{
  MovementAction, MovementCamera, MovementCommand, MovementInput, MovementInputKind, MovementMode, MovementModes,
};
use xrf_game_objects::{ObjectId, ObjectRecord, ObjectRegistry, ObjectSwitching};
use xrf_math::glam::Vec3;
use xrf_physics::PhysicsWorld;
use xrf_spawn::ClsId;
use xrf_world::{WorldFedCamera, WorldFedView, WorldViewFeed};

use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;

/// The level the tests' objects stand on.
const LEVEL: u8 = 108;

fn assert_near(actual: Vec3, expected: Vec3) {
  assert!(actual.abs_diff_eq(expected, 1e-4), "{actual} is not {expected}");
}

/// A new game on a field of objects every 20 m along `+z` on one level, the actor at the origin looking along them.
fn state() -> GameState {
  let mut objects: ObjectRegistry = ObjectRegistry::default();

  for index in 0..50_u16 {
    objects.insert(
      ObjectId(index),
      ObjectRecord {
        section: String::from("stalker"),
        name: format!("stalker_{index}"),
        class: ClsId::AiStlS,
        position: Vec3::new(0.0, 0.0, f32::from(index) * 20.0),
        direction: Vec3::Z,
        level: Some(LEVEL),
        game_vertex: Some(0),
        level_vertex: Some(0),
        parent: None,
        story_id: None,
        spawn_id: index,
        switching: ObjectSwitching::ByDistance,
      },
    );
  }

  GameState {
    objects,
    physics: PhysicsWorld::new().expect("a world"),
    movement: Movement::new(
      MovementCamera::default(),
      MovementMode::default(),
      MovementModes::default(),
    ),
    alife: Alife::new(AlifeSettings::default(), AlifeData::default(), Some(LEVEL)),
  }
}

/// What a replay of a recorded session ends with.
struct Replay {
  frames: Vec<LoopFrame<GameFrame>>,
  objects: ObjectRegistry,
  movement: Movement,
}

fn stepper() -> LoopStepper<GameState> {
  LoopStepper::new(Duration::from_millis(10), GameClock::default(), state())
}

fn input(milliseconds: u64, kind: MovementInputKind) -> GameCommand {
  GameCommand::Movement(MovementCommand::Input(MovementInput {
    at: Duration::from_millis(milliseconds),
    kind,
  }))
}

#[test]
fn routes_a_command_to_its_system_and_publishes_the_camera() {
  let mut stepper: LoopStepper<GameState> = stepper();
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

  let mut stepper: LoopStepper<GameState> = stepper();

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

#[test]
fn switches_objects_online_around_the_actor_as_it_flies() {
  let mut stepper: LoopStepper<GameState> = stepper();

  stepper.step([]);

  // Within 135 m of the origin: the objects at 0 to 120 m.
  assert_eq!(stepper.get_frame().state.online, 7);

  // Fly forward at 10 m/s for 30 s; A-Life sweeps every 265 ms on the way.
  stepper.step([input(
    0,
    MovementInputKind::Action {
      action: MovementAction::Forward,
      is_pressed: true,
    },
  )]);

  for _ in 0..3_000 {
    stepper.step([]);
  }

  let camera_z: f32 = stepper.get_frame().state.camera.position.z;
  let online: Vec<u16> = stepper
    .get_state()
    .objects
    .get_online()
    .iter()
    .map(|(id, _)| id.0)
    .collect();

  // About 300 m along: those within 135 m are online, those beyond 165 m behind are not.
  assert!((camera_z - 300.0).abs() < 1.0, "{camera_z}");
  assert!(online.contains(&15) && online.contains(&21));
  assert!(!online.contains(&0) && !online.contains(&6));
  // 160 m behind is between the two distances: it went online on the way and stays so.
  assert!(online.contains(&7));
}

/// The replay tripwire: a recorded session of flying and looking over a field of objects steps the whole composed state
/// to the same frames, objects and movement every time. A system reading the wall clock, a hash map's order or an
/// unseeded random number breaks it. The physics world is compared through what it moves, which no engine compares
/// whole.
#[test]
fn steps_a_recorded_session_to_the_same_frames_and_state_every_time() {
  let session: Vec<Vec<GameCommand>> = (0..1_000_u64)
    .map(|tick| match tick % 250 {
      0 => vec![input(
        tick * 10,
        MovementInputKind::Action {
          action: MovementAction::Forward,
          is_pressed: true,
        },
      )],
      100 => vec![input(tick * 10, MovementInputKind::Look { dx: 120.0, dy: -15.0 })],
      200 => vec![input(
        tick * 10,
        MovementInputKind::Action {
          action: MovementAction::Accel,
          is_pressed: tick % 500 == 200,
        },
      )],
      _ => Vec::new(),
    })
    .collect();
  let replay = || -> Replay {
    let mut stepper: LoopStepper<GameState> = stepper();
    let frames: Vec<LoopFrame<GameFrame>> = session
      .iter()
      .map(|commands| {
        stepper.step(commands.iter().copied());
        stepper.get_frame()
      })
      .collect();

    Replay {
      frames,
      objects: stepper.get_state().objects.clone(),
      movement: stepper.get_state().movement.clone(),
    }
  };
  let first: Replay = replay();
  let second: Replay = replay();
  let first_frames: &[LoopFrame<GameFrame>] = &first.frames;

  assert_eq!(first.frames, second.frames);
  assert!(first.objects == second.objects);
  assert_eq!(first.movement, second.movement);
  assert!(
    first_frames
      .iter()
      .any(|frame| frame.state.online != first_frames[0].state.online)
  );
}
