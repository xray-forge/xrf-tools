use xrf_game_alife::AlifeContext;
use xrf_game_loop::{LoopState, LoopSystem, LoopTime};
use xrf_game_movement::{Movement, MovementContext};
use xrf_game_objects::ObjectRecord;
use xrf_math::glam::Vec3;

use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;
use crate::state::game_level_state::GameLevelState;
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
  /// then the game's time, then A-Life, which weighs where the player stands after this tick's move.
  fn step(&mut self, time: &LoopTime) {
    let GameProgress {
      objects,
      alife,
      player,
      time: game_time,
      ..
    } = &mut self.progress;

    match objects.get_records_mut().get_mut(*player) {
      Some(record) => self.movement.step(
        time,
        MovementContext {
          physics: &mut self.level.physics,
          player: record,
        },
      ),
      None => log::error!(
        "The game holds no record of its player {}, which does not move",
        player.0
      ),
    }

    if let Err(error) = self.level.physics.step(time.step.as_secs_f32()) {
      log::error!("The physics world did not step: {error}");
    }

    game_time.advance(time.step);

    let actor: Vec3 = objects
      .get_records()
      .get(*player)
      .map_or_else(|| self.movement.get_feet(), |record: &ObjectRecord| record.position);

    alife.step(
      time,
      AlifeContext {
        objects,
        level: self.level.graph_level,
        actor,
      },
    );
  }

  fn get_frame(&self) -> GameFrame {
    let objects = &self.progress.objects;

    GameFrame {
      camera: *self.movement.get_camera(),
      footing: self.movement.get_footing(),
      time_of_day: self.progress.time.get_time_of_day(),
      time_factor: self.progress.time.get_clock().factor,
      objects: u32::try_from(objects.get_records().len()).unwrap_or(u32::MAX),
      online: u32::try_from(objects.get_online().len()).unwrap_or(u32::MAX),
      squads: u32::try_from(objects.get_groups().len()).unwrap_or(u32::MAX),
      members: u32::try_from(objects.get_members().len()).unwrap_or(u32::MAX),
    }
  }
}
