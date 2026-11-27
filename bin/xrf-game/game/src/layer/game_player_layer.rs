use std::time::Duration;

use xrf_error::XrfResult;
use xrf_game_movement::{MovementCommand, MovementInput, MovementInputKind};
use xrf_game_platform::{PlatformEvent, PlatformEventKind};

use crate::layer::game_input_fate::GameInputFate;
use crate::layer::game_layer::GameLayer;
use crate::layer::game_layer_context::GameLayerContext;
use crate::run::game_flow_stage::GameFlowStage;
use crate::state::game_command::GameCommand;

/// The game itself as the bottom layer, taking what no layer over it kept: the mouse turns the look the next drawn
/// frame faces, the keys bound become the loop's movement commands, and each frame the loop is sent the look once it
/// changed, checked for still stepping, and what its steps told is played (the player's footsteps). While the loading
/// view covers the level the player is held, and while a recorded session replays the look follows it.
pub(crate) struct GamePlayerLayer;

impl GameLayer<GameLayerContext<'_>> for GamePlayerLayer {
  fn take_event(&mut self, event: &PlatformEvent, context: &mut GameLayerContext<'_>) -> GameInputFate {
    if context.flow.get_stage() == GameFlowStage::Loading {
      return GameInputFate::Passed;
    }

    // The mouse turns the look at once, which the next drawn frame faces.
    if let PlatformEventKind::MouseMoved { dx, dy, .. } = event.kind {
      let sensitivity: f32 = context.settings.get().input.sensitivity;

      context.flow.get_look().turn(dx, dy, sensitivity);

      return GameInputFate::Kept;
    }

    let command: Option<MovementCommand> = context.flow.get_running().and_then(|running| running.read_input(event));

    match command {
      Some(command) => {
        context.send(GameCommand::Movement(command));

        GameInputFate::Kept
      }
      None => GameInputFate::Passed,
    }
  }

  fn release_input(&mut self, at: Duration, context: &mut GameLayerContext<'_>) {
    if context.flow.get_running().is_some() {
      context.send(GameCommand::Movement(MovementCommand::Input(MovementInput {
        at,
        kind: MovementInputKind::ReleaseAll,
      })));
    }
  }

  fn update(&mut self, context: &mut GameLayerContext<'_>) -> XrfResult<()> {
    context.flow.follow_replay();

    // The view already faces the look the render thread reads; the simulation is sent it whole, once a frame.
    if let Some(heading) = context.flow.get_look().take_changed() {
      context.send(GameCommand::Movement(MovementCommand::Face(heading)));
    }

    if let Some(running) = context.flow.get_running_mut() {
      running.check()?;
      running.play_events();
    }

    Ok(())
  }
}
