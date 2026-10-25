use std::sync::atomic::{AtomicU64, Ordering};

use crate::progress_level::ProgressLevel;
use crate::progress_unit::ProgressUnit;

/// One entered level, while it is entered.
///
/// Held by the code doing that level's work, which is what lets `advance` reach its counter without consulting the
/// stack: a parallel operation would otherwise take a shared lock once per entry to discover where it is.
pub(crate) struct LevelState {
  id: String,
  unit: ProgressUnit,
  total: Option<u64>,
  /// Relaxed throughout: nothing is published through this counter, and a progress snapshot is not a checkpoint.
  completed: AtomicU64,
}

impl LevelState {
  pub(crate) fn new(id: String, unit: ProgressUnit, total: Option<u64>) -> Self {
    Self {
      id,
      unit,
      total,
      completed: AtomicU64::new(0),
    }
  }

  /// Counts `units` more of this level as done.
  pub(crate) fn advance_by(&self, units: u64) {
    self.completed.fetch_add(units, Ordering::Relaxed);
  }

  pub(crate) fn get_completed(&self) -> u64 {
    self.completed.load(Ordering::Relaxed)
  }

  pub(crate) fn describe(&self) -> ProgressLevel {
    ProgressLevel {
      id: self.id.clone(),
      completed: self.get_completed(),
      total: self.total,
      unit: self.unit,
    }
  }
}
