use xrf_game_loop::LoopStepper;
use xrf_game_movement::MovementCamera;

use crate::state::game_state::GameState;

/// Where the game stands as the control endpoint's status tells it, read from the state between two of the loop's
/// ticks rather than published every tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GameStatus {
  pub tick: u64,
  /// Game seconds since midnight.
  pub time_of_day: f32,
  pub objects: usize,
  pub online: usize,
  pub squads: usize,
  pub members: usize,
  pub camera: MovementCamera,
}

impl GameStatus {
  /// The status of the state `stepper` holds.
  pub(crate) fn read(stepper: &LoopStepper<GameState>) -> Self {
    let state: &GameState = stepper.get_state();
    let objects = &state.progress.objects;

    Self {
      tick: stepper.get_time().tick,
      time_of_day: state.progress.time.get_time_of_day(),
      objects: objects.get_records().len(),
      online: objects.get_online().len(),
      squads: objects.get_groups().len(),
      members: objects.get_members().len(),
      camera: *state.movement.get_camera(),
    }
  }
}
