use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::job_handle::JobState;
use crate::progress_level::ProgressLevel;
use crate::progress_unit::ProgressUnit;

/// One entered level, while it is entered.
///
/// Held by the code doing that level's work, which is what lets `advance` reach its counter without consulting the
/// stack: a parallel operation would otherwise take a shared lock once per entry to discover where it is.
pub(crate) struct LevelState {
  pub(crate) id: String,
  pub(crate) label: Option<String>,
  pub(crate) unit: ProgressUnit,
  pub(crate) total: Option<u64>,
  /// Relaxed throughout: nothing is published through this counter, and a progress snapshot is not a checkpoint.
  pub(crate) completed: AtomicU64,
}

impl LevelState {
  pub(crate) fn describe(&self) -> ProgressLevel {
    ProgressLevel {
      id: self.id.clone(),
      label: self.label.clone(),
      completed: self.completed.load(Ordering::Relaxed),
      total: self.total,
      unit: self.unit,
    }
  }
}

/// A level of a job, entered until this is dropped.
///
/// Shared rather than borrowed, so a pool of workers can all advance one level without threading a lifetime through
/// every function the operation is built from.
pub struct JobScope {
  state: Arc<JobState>,
  level: Arc<LevelState>,
}

impl JobScope {
  pub(crate) fn new(state: Arc<JobState>, level: Arc<LevelState>) -> Self {
    Self { state, level }
  }

  /// Count one unit of this level as done.
  ///
  /// Called once per entry from every worker, so the common path is a single relaxed increment plus a counter gate.
  /// Deciding emission here rather than in the caller is what keeps one rule over every operation.
  pub fn advance(&self) {
    self.level.completed.fetch_add(1, Ordering::Relaxed);
    self.state.report_if_due();
  }

  /// Count `units` at once, for a level whose steps are not all one unit — bytes written by a single copy, say.
  pub fn advance_by(&self, units: u64) {
    self.level.completed.fetch_add(units, Ordering::Relaxed);
    self.state.report_if_due();
  }

  /// Enter a level inside this one.
  ///
  /// Offered here as well as on the handle so the nesting reads as nesting at the call site; both do the same thing,
  /// because the stack is the job's rather than any one scope's.
  pub fn enter(&self, id: impl Into<String>, total: Option<u64>) -> JobScope {
    JobState::enter(&self.state, id, total)
  }

  /// How far this level has got, which is what a caller reporting its own result asks for.
  pub fn get_completed(&self) -> u64 {
    self.level.completed.load(Ordering::Relaxed)
  }
}

impl Drop for JobScope {
  /// Leaves the level, counting it against its parent.
  ///
  /// Two emissions rather than one, and intentionally: the first shows this level at its final count, which is the only
  /// moment a finished child is visible at all, and the second shows the parent with that child counted. A boundary is
  /// rare enough that the pair costs nothing, and collapsing them would lose whichever half a reader was watching.
  fn drop(&mut self) {
    self.state.emit();
    self.state.leave(&self.level);
  }
}
