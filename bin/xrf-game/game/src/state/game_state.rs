use xrf_game_alife::{Alife, AlifeContext};
use xrf_game_loop::{LoopState, LoopSystem, LoopTime};
use xrf_game_movement::{Movement, MovementContext};
use xrf_game_objects::ObjectRegistry;
use xrf_physics::PhysicsWorld;

use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;

/// The game's systems put together, the composition root the game loop steps: each command goes to its system, the
/// systems step in this order, and each is handed what it needs of the rest from here, as its context, never looking it
/// up. The object registry and the physics world are the stores they share; adding a system is a field, a command
/// route, and a line in the step below that builds its context.
#[derive(Debug)]
pub struct GameState {
  /// Every object of the game, which systems are handed the tables of they read or write.
  pub objects: ObjectRegistry,
  /// The level's collision and what moves on it, which movement walks the actor in.
  pub physics: PhysicsWorld,
  pub movement: Movement,
  pub alife: Alife,
}

impl LoopState for GameState {
  type Command = GameCommand;
  type Frame = GameFrame;

  fn apply(&mut self, command: GameCommand) {
    match command {
      GameCommand::Movement(command) => self.movement.apply(command),
    }
  }

  /// Movement first, walking the actor in the physics world, then the world's bodies, so A-Life weighs where the actor
  /// stands after this tick's move.
  fn step(&mut self, time: &LoopTime) {
    self.movement.step(
      time,
      MovementContext {
        physics: &mut self.physics,
      },
    );

    if let Err(error) = self.physics.step(time.step.as_secs_f32()) {
      log::error!("The physics world did not step: {error}");
    }

    self.alife.step(
      time,
      AlifeContext {
        objects: &mut self.objects,
        actor: self.movement.get_camera().position,
      },
    );
  }

  fn get_frame(&self) -> GameFrame {
    GameFrame {
      camera: *self.movement.get_camera(),
      objects: u32::try_from(self.objects.get_records().len()).unwrap_or(u32::MAX),
      online: u32::try_from(self.objects.get_online().len()).unwrap_or(u32::MAX),
      squads: u32::try_from(self.objects.get_groups().len()).unwrap_or(u32::MAX),
      members: u32::try_from(self.objects.get_members().len()).unwrap_or(u32::MAX),
    }
  }
}
