use std::time::Instant;

use crate::load::game_load_stage::GameLoadStage;

/// Where one stage of a load stands: when it began and ended, and how far along it is between, from 0 to 1, for a stage
/// that can tell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GameLoadStageState {
  pub(crate) stage: GameLoadStage,
  pub(crate) began: Option<Instant>,
  pub(crate) ended: Option<Instant>,
  pub(crate) fraction: f32,
}

impl GameLoadStageState {
  /// How much of the load it counts for: whole once ended, its fraction while it runs, nothing before.
  pub(crate) fn get_share(&self) -> f32 {
    if self.ended.is_some() {
      1.0
    } else if self.began.is_some() {
      self.fraction.clamp(0.0, 1.0)
    } else {
      0.0
    }
  }

  pub(crate) fn is_running(&self) -> bool {
    self.began.is_some() && self.ended.is_none()
  }
}
