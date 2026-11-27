use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_game_alife::{Alife, AlifeData, AlifeSettings};
use xrf_game_loop::{GameLoop, LoopClock, LoopFeed, LoopFrame, LoopStepper};
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
use crate::state::game_level_entry::GameLevelEntry;
use crate::state::game_level_state::GameLevelState;
use crate::state::game_progress::GameProgress;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;
use crate::time::game_clock::GameClock;
use crate::time::game_time::GameTime;

/// The level the tests' objects stand on.
const LEVEL: u8 = 108;

/// The player's object, past the field of stalkers.
const PLAYER: ObjectId = ObjectId(50);

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
    level: Some(LEVEL),
    game_vertex: Some(0),
    level_vertex: Some(0),
    parent: None,
    story_id: None,
    spawn_id,
    switching: ObjectSwitching::ByDistance,
  }
}

/// A new game on a field of objects every 20 m along `+z` on one level, the player at the origin looking along them.
fn state() -> GameState {
  let mut objects: ObjectRegistry = ObjectRegistry::default();

  for index in 0..50_u16 {
    objects.insert(
      ObjectId(index),
      record(
        format!("stalker_{index}"),
        ClsId::AiStlS,
        Vec3::new(0.0, 0.0, f32::from(index) * 20.0),
        index,
      ),
    );
  }

  objects.insert(PLAYER, record(String::from("actor"), ClsId::SActor, Vec3::ZERO, 50));

  GameState {
    progress: GameProgress {
      seed: 7,
      time: GameTime::new(GameClock::default()),
      objects,
      alife: Alife::new(AlifeSettings::default(), AlifeData::default()),
      player: PLAYER,
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

  stepper.step([GameCommand::Movement(MovementCommand::SetCamera(camera))]);

  assert_eq!(stepper.get_frame().state.camera, camera);
  assert_eq!(*stepper.get_state().movement.get_camera(), camera);
}

#[test]
fn runs_the_games_own_time_by_its_ticks() {
  let mut stepper: LoopStepper<GameState> = stepper();
  let start: f32 = stepper.get_frame().state.time_of_day;

  for _ in 0..100 {
    stepper.step([]);
  }

  // A simulated second at ten game seconds a second.
  assert_eq!(stepper.get_state().progress.time.get_elapsed(), 10_000);
  assert!((stepper.get_frame().state.time_of_day - start - 10.0).abs() < 1e-3);
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

  assert_near(read.camera.position, Vec3::new(4.0, 5.0, 6.0));
  assert_near(read.camera.forward, Vec3::X);
  assert_near(read.camera.up, Vec3::Y);
  assert_eq!(read.camera.field_of_view, 75.0);

  let clock = read.clock.expect("the game runs the day");

  assert!((clock.time - stepper.get_state().progress.time.get_time_of_day()).abs() < 1e-3);
  assert_eq!(clock.rate, 10.0);
}

#[test]
fn runs_the_day_forward_across_midnight_between_frames() {
  let frame = |time_of_day: f32| GameFrame {
    camera: MovementCamera::default(),
    footing: None,
    time_of_day,
    time_factor: 10.0,
    objects: 0,
    online: 0,
    squads: 0,
    members: 0,
  };
  let blended: GameFrame = xrf_game_loop::LoopBlend::blend(&frame(86_399.0), &frame(1.0), 0.5);

  assert!(
    blended.time_of_day.abs() < 1e-2,
    "{} is not midnight",
    blended.time_of_day
  );
}

#[test]
fn switches_objects_online_around_the_player_as_it_flies_and_writes_its_record() {
  let mut stepper: LoopStepper<GameState> = stepper();

  stepper.step([]);

  // Within 135 m of the origin: the objects at 0 to 120 m, and the player.
  assert_eq!(stepper.get_frame().state.online, 8);

  // Fly forward at 10 m/s for 30 s; A-Life sweeps every 265 ms on the way.
  stepper.step([press_forward()]);

  for _ in 0..3_000 {
    stepper.step([]);
  }

  let camera_z: f32 = stepper.get_frame().state.camera.position.z;
  let progress: &GameProgress = &stepper.get_state().progress;
  let online: Vec<u16> = progress.objects.get_online().iter().map(|(id, _)| id.0).collect();
  let player: &ObjectRecord = progress.objects.get_records().get(PLAYER).expect("the player's record");

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
        stepper.step(commands.iter().copied().map(GameCommand::Movement));
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
      .any(|frame| frame.state.online != first_frames[0].state.online)
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
