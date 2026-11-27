use xrf_error::{XrfError, XrfResult};
use xrf_game_alife::AlifeContext;
use xrf_game_loop::{LoopState, LoopSystem, LoopTime};
use xrf_game_movement::{Movement, MovementContext, MovementEvent};
use xrf_game_objects::{ObjectRecord, ObjectRegistry};
use xrf_math::glam::Vec3;

use crate::state::game_command::GameCommand;
use crate::state::game_event::GameEvent;
use crate::state::game_frame::GameFrame;
use crate::state::game_level_state::GameLevelState;
use crate::state::game_pose::GamePose;
use crate::state::game_progress::GameProgress;

/// The game's systems put together, the composition root the game loop steps: each command goes to its system, the
/// systems step in this order, and each is handed what it needs of the rest from here, as its context, never looking it
/// up. What it holds lives as long as one of two things: the game (`progress`, what a save holds, and the player's
/// `movement`) or the level played (`level`, replaced whole when the game moves to another). Adding a system is a field
/// in the lifetime it shares, a command route, and a line in the step below that builds its context.
#[derive(Debug)]
pub struct GameState {
  pub progress: GameProgress,
  pub level: GameLevelState,
  pub movement: Movement,
}

impl LoopState for GameState {
  type Command = GameCommand;
  type Frame = GameFrame;
  type Event = GameEvent;

  fn apply(&mut self, command: GameCommand) {
    match command {
      GameCommand::Movement(command) => self.movement.apply(command),
      GameCommand::ReplaceLevel(entry) => {
        log::info!(
          "Game moves to graph level {:?}, {} ms of game time in",
          entry.level.graph_level,
          self.progress.time.get_elapsed()
        );
        self.level = entry.level;
        self.movement.enter_level(entry.start);
      }
    }
  }

  /// Movement first, walking the player in the level's physics world and writing its record, then the world's bodies,
  /// then the game's time, then A-Life, which weighs where the player stands after this tick's move. What each system
  /// tells goes to `events`, routed as the game's.
  fn step(&mut self, time: &LoopTime, events: &mut Vec<GameEvent>) -> XrfResult<()> {
    let GameProgress {
      objects,
      alife,
      player,
      time: game_time,
      ..
    } = &mut self.progress;
    let record: &mut ObjectRecord = objects
      .get_records_mut()
      .get_mut(*player)
      .ok_or_else(|| XrfError::new_unexpected_error(format!("the game holds no record of its player {}", player.0)))?;
    let mut movement_events: Vec<MovementEvent> = Vec::new();

    self.movement.step(
      time,
      MovementContext {
        physics: &mut self.level.physics,
        player: record,
        events: &mut movement_events,
      },
    )?;
    events.extend(movement_events.into_iter().map(GameEvent::Movement));
    self.level.physics.step(time.step.as_secs_f32())?;
    game_time.advance(time.step);

    let actor: Vec3 = record.position;

    alife.step(
      time,
      AlifeContext {
        objects,
        level: self.level.graph_level,
        actor,
      },
    )
  }

  fn get_frame(&self) -> GameFrame {
    let objects: &ObjectRegistry = &self.progress.objects;

    GameFrame {
      camera: *self.movement.get_camera(),
      velocity: self.movement.get_velocity(),
      time_of_day: self.progress.time.get_time_of_day(),
      time_factor: self.progress.time.get_clock().factor,
      poses: objects
        .get_online()
        .iter()
        .filter_map(|(id, _)| {
          let record: &ObjectRecord = objects.get_records().get(id)?;

          Some(GamePose {
            id,
            position: record.position,
            direction: record.direction,
          })
        })
        .collect(),
    }
  }
}
