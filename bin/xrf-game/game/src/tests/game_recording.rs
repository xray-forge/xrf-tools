use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use xrf_game_loop::{LoopCommandLog, LoopSession, LoopStepper, LoopWitness};
use xrf_game_movement::{
  Movement, MovementAction, MovementCommand, MovementEvent, MovementHeading, MovementInput, MovementInputKind,
  MovementMode,
};
use xrf_math::glam::Vec3;

use crate::record::game_recorded_command::GameRecordedCommand;
use crate::record::game_recorder::GameRecorder;
use crate::record::game_recording_header::GameRecordingHeader;
use crate::record::game_recording_line::GameRecordingLine;
use crate::record::game_replay::GameReplay;
use crate::record::game_replay_check::GameReplayCheck;
use crate::record::game_state_digest::GameStateDigest;
use crate::state::game_command::GameCommand;
use crate::state::game_event::GameEvent;
use crate::state::game_progress::GameProgress;
use crate::state::game_state::GameState;
use crate::tests::fixtures::{player_id, walking_state};

/// How many ticks the recorded walk lasts.
const TICKS: u64 = 1_500;

fn press(action: MovementAction, is_pressed: bool) -> MovementCommand {
  MovementCommand::Input(MovementInput {
    at: Duration::ZERO,
    kind: MovementInputKind::Action { action, is_pressed },
  })
}

/// The commands `tick` of a walk takes: forward from the start, a turn every second by an angle no decimal writes
/// short, then a walk held, a sprint, a jump, a flight and a landing.
fn walk_at(tick: u64) -> Vec<MovementCommand> {
  let mut commands: Vec<MovementCommand> = match tick {
    0 => vec![press(MovementAction::Forward, true)],
    250 => vec![press(MovementAction::Accel, true), press(MovementAction::Right, true)],
    400 => vec![press(MovementAction::Accel, false), press(MovementAction::Right, false)],
    500 => vec![press(MovementAction::Sprint, true)],
    510 => vec![press(MovementAction::Sprint, false)],
    700 => vec![press(MovementAction::Up, true)],
    705 => vec![press(MovementAction::Up, false)],
    900 => vec![MovementCommand::ToggleMode, press(MovementAction::Up, true)],
    1_000 => vec![press(MovementAction::Up, false), MovementCommand::ToggleMode],
    1_300 => vec![press(MovementAction::Down, true)],
    _ => Vec::new(),
  };

  if tick % 100 == 50 {
    commands.push(MovementCommand::Face(MovementHeading {
      yaw: (tick as f32).sqrt() / 7.0,
      pitch: -1.0 / 3.0,
    }));
  }

  commands
}

fn commands_at(tick: u64) -> Vec<GameCommand> {
  walk_at(tick).into_iter().map(GameCommand::Movement).collect()
}

fn stepper() -> LoopStepper<GameState> {
  LoopStepper::new(Duration::from_millis(10), walking_state())
}

/// What a walk ends with, and what it told and came to on the way.
struct Walked {
  digests: Vec<GameStateDigest>,
  strides: usize,
  progress: GameProgress,
  movement: Movement,
}

/// Steps the walk from a new game, its state's digest taken at each interval.
fn walk() -> Walked {
  let mut stepper: LoopStepper<GameState> = stepper();
  let mut digests: Vec<GameStateDigest> = Vec::new();
  let mut strides: usize = 0;

  for tick in 0..TICKS {
    let events: Vec<GameEvent> = stepper.step(commands_at(tick)).expect("a step");

    strides += events
      .iter()
      .filter(|event| matches!(event, GameEvent::Movement(MovementEvent::Step(_))))
      .count();

    if stepper.get_time().tick.is_multiple_of(GameStateDigest::INTERVAL) {
      digests.push(GameStateDigest::of(stepper.get_time().tick, stepper.get_state()));
    }
  }

  Walked {
    digests,
    strides,
    progress: stepper.get_state().progress.clone(),
    movement: stepper.get_state().movement.clone(),
  }
}

/// A file for a test's recording, apart from every other test's and run's.
fn recording_file(name: &str) -> PathBuf {
  std::env::temp_dir().join(format!("xrf-game-recording-{}-{name}.jsonl", std::process::id()))
}

fn header() -> GameRecordingHeader {
  GameRecordingHeader {
    version: GameRecordingHeader::VERSION,
    level: String::from("zaton"),
    seed: 7,
    mode: MovementMode::Walk,
    tick_rate: 100,
  }
}

/// Records the walk to `file` as the loop's thread would: each tick's commands seen, the tick stepped, the step seen.
fn record_walk(file: &Path) {
  let mut recorder: GameRecorder = GameRecorder::create(file, &header()).expect("the recording starts");
  let mut stepper: LoopStepper<GameState> = stepper();

  for tick in 0..TICKS {
    let commands: Vec<GameCommand> = commands_at(tick);

    recorder.see_commands(tick, &commands);
    stepper.step(commands).expect("a step");
    recorder.see_step(&stepper);
  }
}

/// Steps a new game through `log` to its end, each tick compared by `check`.
fn step_through(mut log: LoopCommandLog<GameCommand>, check: &mut GameReplayCheck) {
  let mut stepper: LoopStepper<GameState> = stepper();

  for tick in 0..log.get_end() {
    let commands: Vec<GameCommand> = log.take_at(tick).collect();

    stepper.step(commands).expect("a step");
    check.compare(stepper.get_time().tick, stepper.get_state());
  }
}

/// What a recording's file holds, read a line at a time.
struct Lines {
  log: LoopCommandLog<GameCommand>,
  digests: BTreeMap<u64, String>,
}

fn read_lines(file: &Path) -> Lines {
  let mut lines: Lines = Lines {
    log: LoopCommandLog::default(),
    digests: BTreeMap::new(),
  };

  for line in std::fs::read_to_string(file).expect("the recording reads").lines() {
    match serde_json::from_str::<GameRecordingLine>(line).expect("a recording's line") {
      GameRecordingLine::Header(read) => assert_eq!(read, header()),
      GameRecordingLine::Command { tick, command } => lines.log.push(tick, command.into()).expect("in order"),
      GameRecordingLine::Digest { tick, digest } => {
        lines.digests.insert(tick, digest);
      }
      GameRecordingLine::End { tick } => lines.log.end_at(tick),
    }
  }

  lines
}

/// The replay tripwire on foot: a recorded walk over a floor, the physics engine stepped every tick, comes to the same
/// state every time, by its digests and by the state itself. A physics engine that does not repeat, a system reading
/// the wall clock, a hash map's order or an unseeded random number breaks it.
#[test]
fn walks_a_recorded_session_on_the_physics_to_the_same_digests_and_state_every_time() {
  let first: Walked = walk();
  let second: Walked = walk();

  assert_eq!(first.digests.len() as u64, TICKS / GameStateDigest::INTERVAL);
  assert_eq!(first.digests, second.digests);
  assert!(first.progress == second.progress);
  // By its text, as the digest reads it: the walking actor's id remembers its physics world, so the ids of two worlds
  // never compare equal though they name the same actor.
  assert_eq!(format!("{:?}", first.movement), format!("{:?}", second.movement));

  // The walk went somewhere on its feet, so what repeated is the physics' work, and the state moved between digests.
  let player: Vec3 = first
    .progress
    .objects
    .get_records()
    .get(player_id())
    .expect("the player's record")
    .position;

  assert!(player.with_y(0.0).length() > 10.0, "{player}");
  assert!(first.strides > 10, "{} strides", first.strides);
  assert!(first.digests.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn tells_two_states_apart_by_one_value() {
  let mut stepper: LoopStepper<GameState> = stepper();
  let before: GameStateDigest = GameStateDigest::of(0, stepper.get_state());

  assert_eq!(before, GameStateDigest::of(0, stepper.get_state()));
  assert_ne!(before, GameStateDigest::of(1, stepper.get_state()));
  assert_eq!(before.to_string().len(), 16);

  stepper
    .step([GameCommand::Movement(press(MovementAction::Forward, true))])
    .expect("a step");

  // The tick aside, the held key alone parts them.
  assert_ne!(
    GameStateDigest::of(0, stepper.get_state()),
    GameStateDigest::of(0, self::stepper().get_state())
  );
}

#[test]
fn records_a_walk_to_a_file_and_replays_it_to_every_digest() {
  let file: PathBuf = recording_file("replays");

  record_walk(&file);

  let recorded: Lines = read_lines(&file);
  let replayed: GameReplay = GameReplay::read(&file).expect("the recording reads back");
  let commands: usize = (0..TICKS).map(|tick| walk_at(tick).len()).sum();

  // The file holds every command at its tick, a digest each interval, and where the session stopped.
  assert_eq!(*replayed.get_header(), header());
  assert_eq!(GameReplay::read_header(&file).expect("the header reads"), header());
  assert_eq!(replayed.get_end(), TICKS);
  assert_eq!(recorded.log.len(), commands);
  assert_eq!(recorded.digests.len() as u64, TICKS / GameStateDigest::INTERVAL);

  let LoopSession { replay, witness, .. } = replayed.into_session();
  let log: LoopCommandLog<GameCommand> = replay.expect("a replay's commands");

  assert!(witness.is_some());
  assert_eq!(log.len(), commands);
  assert_eq!(log.get_end(), TICKS);

  // Stepped again from a new game through the file's own lines, headings as their text reads back, the state follows
  // the recording digest for digest.
  let mut check: GameReplayCheck = GameReplayCheck::new(recorded.digests.clone(), TICKS);

  step_through(log, &mut check);

  assert_eq!(check.get_parted_at(), None);
  assert_eq!(check.count_matched(), recorded.digests.len());

  std::fs::remove_file(&file).expect("the recording is removed");
}

#[test]
fn says_where_a_replay_parted_from_its_recording() {
  let file: PathBuf = recording_file("parts");

  record_walk(&file);

  let recorded: Lines = read_lines(&file);
  let mut log: LoopCommandLog<GameCommand> = LoopCommandLog::default();

  // The same walk, but for a turn at tick 320 the recording never took.
  for tick in 0..TICKS {
    for command in walk_at(tick) {
      log.push(tick, GameCommand::Movement(command)).expect("in order");
    }

    if tick == 320 {
      log
        .push(
          tick,
          GameCommand::Movement(MovementCommand::Face(MovementHeading { yaw: 2.0, pitch: 0.0 })),
        )
        .expect("in order");
    }
  }

  log.end_at(TICKS);

  let mut check: GameReplayCheck = GameReplayCheck::new(recorded.digests, TICKS);

  step_through(log, &mut check);

  // The three digests before the turn matched; the first after it did not, and nothing is compared past it.
  assert_eq!(check.get_parted_at(), Some(400));
  assert_eq!(check.count_matched(), 3);

  std::fs::remove_file(&file).expect("the recording is removed");
}

#[test]
fn refuses_a_recording_of_another_format_or_without_a_header() {
  let other: PathBuf = recording_file("other-format");
  let headless: PathBuf = recording_file("headless");
  let line = |line: &GameRecordingLine| serde_json::to_string(line).expect("a line");

  std::fs::write(
    &other,
    line(&GameRecordingLine::Header(GameRecordingHeader {
      version: GameRecordingHeader::VERSION + 1,
      ..header()
    })),
  )
  .expect("a file");
  std::fs::write(
    &headless,
    line(&GameRecordingLine::Command {
      tick: 0,
      command: GameRecordedCommand::Movement(MovementCommand::ToggleMode),
    }),
  )
  .expect("a file");

  assert!(GameReplay::read(&other).is_err());
  assert!(GameReplay::read_header(&headless).is_err());
  assert!(GameReplay::read(&recording_file("missing")).is_err());

  std::fs::remove_file(&other).expect("the recording is removed");
  std::fs::remove_file(&headless).expect("the recording is removed");
}
