use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_game_alife::{Alife, AlifeData, AlifeHooks, AlifeSettings};
use xrf_game_loop::{GameLoop, LoopClock, LoopFeed, LoopFrame, LoopRead, LoopStepper};
use xrf_game_movement::Movement;
use xrf_game_movement::{
  MovementAction, MovementCamera, MovementCommand, MovementHeading, MovementInput, MovementInputKind, MovementMode,
  MovementModes,
};
use xrf_game_objects::{ObjectId, ObjectRecord, ObjectRegistry, ObjectSwitching};
use xrf_math::glam::Vec3;
use xrf_physics::PhysicsWorld;
use xrf_spawn::ClsId;
use xrf_world::{WorldFedCamera, WorldFedView, WorldViewFeed};

use xrf_game_settings::GameInputSettings;

use crate::input::game_look::GameLook;
use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;
use crate::state::game_level_entry::GameLevelEntry;
use crate::state::game_level_state::GameLevelState;
use crate::state::game_progress::GameProgress;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;
use crate::time::game_clock::GameClock;
use crate::time::game_time::GameTime;

/// The level the tests' objects stand on.
const LEVEL: u8 = 108;

/// How many stalkers stand in the field.
const STALKERS: u16 = 50;

/// The player's object, past the field of stalkers.
fn player_id() -> ObjectId {
  ObjectId::of_index(u32::from(STALKERS))
}

/// Longer than any of these waits should take, so a broken contract fails rather than hangs.
const WAIT: Duration = Duration::from_secs(10);

fn assert_near(actual: Vec3, expected: Vec3) {
  assert!(actual.abs_diff_eq(expected, 1e-4), "{actual} is not {expected}");
}

fn record(name: String, class: ClsId, position: Vec3, spawn_id: u16) -> ObjectRecord {
  ObjectRecord {
    section: String::from("stalker"),
    name,
    class,
    position,
    direction: Vec3::Z,
    game_vertex: Some(0),
    level_vertex: Some(0),
    story_id: None,
    spawn_id,
    switching: ObjectSwitching::ByDistance,
  }
}

/// A new game on a field of objects every 20 m along `+z` on one level, the player at the origin looking along them.
fn state() -> GameState {
  let mut objects: ObjectRegistry = ObjectRegistry::default();

  for index in 0..STALKERS {
    objects.create(
      record(
        format!("stalker_{index}"),
        ClsId::AiStlS,
        Vec3::new(0.0, 0.0, f32::from(index) * 20.0),
        index,
      ),
      Some(LEVEL),
    );
  }

  let player: Option<ObjectId> = objects.create(
    record(String::from("actor"), ClsId::SActor, Vec3::ZERO, STALKERS),
    Some(LEVEL),
  );

  assert_eq!(player, Some(player_id()));

  GameState {
    progress: GameProgress {
      seed: 7,
      time: GameTime::new(GameClock::default()),
      objects,
      alife: Alife::new(AlifeSettings::default(), AlifeData::default()),
      player: player_id(),
    },
    level: GameLevelState {
      physics: PhysicsWorld::new().expect("a world"),
      graph_level: Some(LEVEL),
    },
    movement: Movement::new(
      MovementCamera::default(),
      MovementMode::default(),
      MovementModes::default(),
    ),
    hooks: AlifeHooks::of_defaults().expect("A-Life's hooks"),
  }
}

/// What a replay of a recorded session ends with.
struct Replay {
  frames: Vec<LoopFrame<GameFrame>>,
  progress: GameProgress,
  movement: Movement,
}

fn stepper() -> LoopStepper<GameState> {
  LoopStepper::new(Duration::from_millis(10), state())
}

fn input(milliseconds: u64, kind: MovementInputKind) -> MovementCommand {
  MovementCommand::Input(MovementInput {
    at: Duration::from_millis(milliseconds),
    kind,
  })
}

fn press_forward() -> GameCommand {
  GameCommand::Movement(input(
    0,
    MovementInputKind::Action {
      action: MovementAction::Forward,
      is_pressed: true,
    },
  ))
}

#[test]
fn routes_a_command_to_its_system_and_publishes_the_camera() {
  let mut stepper: LoopStepper<GameState> = stepper();
  let camera: MovementCamera = MovementCamera::looking_at(Vec3::new(1.0, 2.0, 3.0), Vec3::new(1.0, 2.0, 9.0));

  stepper
    .step([GameCommand::Movement(MovementCommand::SetCamera(camera))])
    .expect("a step");

  assert_eq!(stepper.get_frame().state.camera, camera);
  assert_eq!(*stepper.get_state().movement.get_camera(), camera);
}

#[test]
fn runs_the_games_own_time_by_its_ticks() {
  let mut stepper: LoopStepper<GameState> = stepper();
  let start: f32 = stepper.get_frame().state.time_of_day;

  for _ in 0..100 {
    stepper.step([]).expect("a step");
  }

  // A simulated second at ten game seconds a second.
  assert_eq!(stepper.get_state().progress.time.get_elapsed(), 10_000);
  assert!((stepper.get_frame().state.time_of_day - start - 10.0).abs() < 1e-3);
}

#[test]
fn feeds_the_world_the_latest_tick_facing_the_look_through_its_lens_and_the_games_day() {
  let feed: Arc<LoopFeed<GameFrame>> = Arc::new(LoopFeed::default());
  let look: Arc<GameLook> = Arc::new(GameLook::default());
  let lens: WorldFedCamera = WorldFedCamera {
    field_of_view: 75.0,
    ..WorldFedCamera::default()
  };
  let view: GameViewFeed = GameViewFeed::new(Arc::clone(&feed), Arc::clone(&look), lens);
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

  stepper
    .step([GameCommand::Movement(MovementCommand::SetCamera(
      MovementCamera::looking_at(Vec3::new(4.0, 5.0, 6.0), Vec3::new(5.0, 5.0, 6.0)),
    ))])
    .expect("a step");
  feed.publish(now, stepper.get_frame());
  // Presentation turned the look a quarter since: the view faces it at once, before any tick took it.
  look.face(MovementHeading {
    yaw: -FRAC_PI_2,
    pitch: 0.0,
  });

  let read: WorldFedView = view.read(now);

  assert_near(read.camera.position, Vec3::new(4.0, 5.0, 6.0));
  assert_near(read.camera.forward, -Vec3::X);
  assert_near(read.camera.up, Vec3::Y);
  assert_eq!(read.camera.field_of_view, 75.0);

  let clock = read.clock.expect("the game runs the day");

  assert!((clock.time - stepper.get_state().progress.time.get_time_of_day()).abs() < 1e-3);
  assert_eq!(clock.rate, 10.0);
}

#[test]
fn carries_the_players_view_forward_from_its_tick_by_how_fast_it_moved() {
  let feed: Arc<LoopFeed<GameFrame>> = Arc::new(LoopFeed::default());
  let view: GameViewFeed = GameViewFeed::new(
    Arc::clone(&feed),
    Arc::new(GameLook::default()),
    WorldFedCamera::default(),
  );
  let mut stepper: LoopStepper<GameState> = stepper();
  let at: Instant = Instant::now();

  // Flying forward along `+z` at 10 m/s.
  stepper.step([press_forward()]).expect("a step");
  stepper.step([]).expect("a step");
  feed.publish(at, stepper.get_frame());

  let tick: Vec3 = stepper.get_frame().state.camera.position;
  let read = |after: Duration| -> Vec3 {
    let read: LoopRead<GameFrame> = feed.read(at + after).expect("a frame");

    view.get_camera(&read, at + after).position
  };

  assert_near(read(Duration::ZERO), tick);
  // 5 ms after its tick, 5 cm further on.
  assert_near(read(Duration::from_millis(5)), tick + Vec3::new(0.0, 0.0, 0.05));
  // A stalled loop holds the view 50 ms on rather than drifting away.
  assert_near(read(Duration::from_secs(1)), tick + Vec3::new(0.0, 0.0, 0.5));
}

#[test]
fn runs_the_day_forward_across_midnight_between_frames() {
  let frame = |time_of_day: f32| GameFrame {
    camera: MovementCamera::default(),
    velocity: Vec3::ZERO,
    time_of_day,
    time_factor: 10.0,
    poses: Vec::new(),
  };
  let time: f32 = frame(86_399.0).blend_time_of_day(&frame(1.0), 0.5);

  assert!(time.abs() < 1e-2, "{time} is not midnight");
}

#[test]
fn turns_the_look_as_the_mouse_moves_and_sends_it_once_a_change() {
  let look: GameLook = GameLook::default();
  let sensitivity: f32 = GameInputSettings::SENSITIVITY;
  let quarter: f32 = FRAC_PI_2 / sensitivity;

  // Moved right turns right: a quarter turn faces `+x`.
  look.turn(quarter, 0.0, sensitivity);
  assert!(look.get().get_rotation().mul_vec3(Vec3::Z).abs_diff_eq(Vec3::X, 1e-4));
  assert_eq!(look.take_changed(), Some(look.get()));
  assert_eq!(look.take_changed(), None);

  // Moved down looks down, held just short of straight down.
  look.turn(0.0, quarter * 3.0, sensitivity);
  assert!((look.get().pitch - MovementHeading::MOST_PITCH).abs() < 1e-4);

  // Facing where the simulation was stood sends nothing back.
  look.face(MovementHeading::default());
  assert_eq!(look.take_changed(), None);
}

#[test]
fn switches_objects_online_around_the_player_as_it_flies_and_writes_its_record() {
  let mut stepper: LoopStepper<GameState> = stepper();

  stepper.step([]).expect("a step");

  // Within 135 m of the origin: the objects at 0 to 120 m, and the player, posed in id order.
  let poses: Vec<u32> = stepper
    .get_frame()
    .state
    .poses
    .iter()
    .map(|pose| pose.id.get_index())
    .collect();

  assert_eq!(poses, vec![0, 1, 2, 3, 4, 5, 6, 50]);

  // Fly forward at 10 m/s for 30 s; A-Life sweeps every 265 ms on the way.
  stepper.step([press_forward()]).expect("a step");

  for _ in 0..3_000 {
    stepper.step([]).expect("a step");
  }

  let camera_z: f32 = stepper.get_frame().state.camera.position.z;
  let progress: &GameProgress = &stepper.get_state().progress;
  let online: Vec<u32> = progress
    .objects
    .get_online()
    .iter()
    .map(|(id, _)| id.get_index())
    .collect();
  let player: &ObjectRecord = progress
    .objects
    .get_records()
    .get(player_id())
    .expect("the player's record");

  // About 300 m along: those within 135 m are online, those beyond 165 m behind are not.
  assert!((camera_z - 300.0).abs() < 1.0, "{camera_z}");
  assert!(online.contains(&15) && online.contains(&21));
  assert!(!online.contains(&0) && !online.contains(&6));
  // 160 m behind is between the two distances: it went online on the way and stays so.
  assert!(online.contains(&7));
  // The player's record went with it: in flight, where the camera is.
  assert_near(player.position, stepper.get_frame().state.camera.position);
}

/// The replay tripwire: a recorded session of flying and looking over a field of objects steps the whole composed state
/// to the same frames, objects and movement every time. A system reading the wall clock, a hash map's order or an
/// unseeded random number breaks it. The physics world is compared through what it moves, which no engine compares
/// whole.
#[test]
fn steps_a_recorded_session_to_the_same_frames_and_state_every_time() {
  let session: Vec<Vec<MovementCommand>> = (0..1_000_u64)
    .map(|tick| match tick % 250 {
      0 => vec![input(
        tick * 10,
        MovementInputKind::Action {
          action: MovementAction::Forward,
          is_pressed: true,
        },
      )],
      100 => vec![MovementCommand::Face(MovementHeading {
        yaw: tick as f32 * 0.001,
        pitch: -0.04,
      })],
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
        stepper
          .step(commands.iter().copied().map(GameCommand::Movement))
          .expect("a step");
        stepper.get_frame()
      })
      .collect();

    Replay {
      frames,
      progress: stepper.get_state().progress.clone(),
      movement: stepper.get_state().movement.clone(),
    }
  };
  let first: Replay = replay();
  let second: Replay = replay();
  let first_frames: &[LoopFrame<GameFrame>] = &first.frames;

  assert_eq!(first.frames, second.frames);
  assert!(first.progress == second.progress);
  assert_eq!(first.movement, second.movement);
  assert!(
    first_frames
      .iter()
      .any(|frame| frame.state.poses.len() != first_frames[0].state.poses.len())
  );
}

/// What the tests read of the running loop's state between two of its ticks.
struct Seen {
  tick: u64,
  graph_level: Option<u8>,
  camera: MovementCamera,
  objects: usize,
  elapsed: u64,
}

fn read(game_loop: &GameLoop<GameState>) -> Seen {
  game_loop
    .snapshot(|stepper| {
      let state: &GameState = stepper.get_state();

      Seen {
        tick: stepper.get_time().tick,
        graph_level: state.level.graph_level,
        camera: *state.movement.get_camera(),
        objects: state.progress.objects.get_records().len(),
        elapsed: state.progress.time.get_elapsed(),
      }
    })
    .recv_timeout(WAIT)
    .expect("the loop answers a snapshot")
}

/// Reads the running loop until `done` holds of what it read.
fn read_until(game_loop: &GameLoop<GameState>, done: impl Fn(&Seen) -> bool) -> Seen {
  let deadline: Instant = Instant::now() + WAIT;

  loop {
    let seen: Seen = read(game_loop);

    if done(&seen) {
      return seen;
    }

    assert!(Instant::now() < deadline, "the loop never reached what was waited for");
    std::thread::sleep(Duration::from_millis(5));
  }
}

#[test]
fn replaces_the_level_under_a_running_loop_keeping_the_game() {
  let clock: LoopClock = LoopClock::new(Instant::now(), LoopClock::DEFAULT_TICK_RATE);
  let game_loop: GameLoop<GameState> =
    GameLoop::start(LoopStepper::new(clock.get_step(), state()), clock).expect("the loop starts");
  let before: Seen = read_until(&game_loop, |seen| seen.tick > 5);
  let start: MovementCamera = MovementCamera::looking_at(Vec3::new(30.0, 4.0, -7.0), Vec3::new(30.0, 4.0, 1.0));
  // The next level, built beside the running loop as the pool builds one.
  let entry: GameLevelEntry = GameLevelEntry {
    level: GameLevelState {
      physics: PhysicsWorld::new().expect("a world"),
      graph_level: Some(LEVEL + 1),
    },
    start,
  };

  assert!(
    game_loop
      .get_sender()
      .send(GameCommand::ReplaceLevel(Box::new(entry)))
      .is_ok()
  );

  let after: Seen = read_until(&game_loop, |seen| seen.graph_level == Some(LEVEL + 1));

  // The level and where the player stands are the new level's; the game and its time went on.
  assert_eq!(after.camera, start);
  assert_eq!(after.objects, before.objects);
  assert!(after.tick > before.tick);
  assert!(after.elapsed > before.elapsed);

  // The loop steps on over the new level.
  let later: Seen = read_until(&game_loop, |seen| seen.tick > after.tick + 5);

  assert_eq!(later.graph_level, Some(LEVEL + 1));
}
