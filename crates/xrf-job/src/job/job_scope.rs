use std::sync::Arc;

use crate::job::JobState;
use crate::job::LevelState;

/// A level of a job, entered until this is dropped.
pub struct JobScope {
  state: Arc<JobState>,
  level: Arc<LevelState>,
}

impl JobScope {
  pub(crate) fn new(state: Arc<JobState>, level: Arc<LevelState>) -> Self {
    Self { state, level }
  }

  /// Count one unit of this level as done.
  pub fn advance(&self) {
    self.advance_by(1);
  }

  /// Count `units` at once, for a level whose steps are not all one unit — bytes written by a single copy, say.
  pub fn advance_by(&self, units: u64) {
    self.level.advance_by(units);
    self.state.report_if_due();
  }

  /// Enter a level inside this one.
  pub fn enter(&self, id: impl Into<String>, total: Option<u64>) -> JobScope {
    JobState::enter(&self.state, id, total)
  }

  /// How far this level has got, which is what a caller reporting its own result asks for.
  pub fn get_completed(&self) -> u64 {
    self.level.get_completed()
  }
}

impl Drop for JobScope {
  /// Leaves the level. The level it was entered in counts only what its own owner advances.
  fn drop(&mut self) {
    self.state.emit();
    self.state.leave(&self.level);
  }
}
