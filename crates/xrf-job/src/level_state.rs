use std::sync::atomic::{AtomicU64, Ordering};

use crate::progress_level::ProgressLevel;
use crate::progress_unit::ProgressUnit;

/// One entered level, while it is entered.
///
/// Held by the code doing that level's work, which is what lets `advance` reach its counter without consulting the
/// stack: a parallel operation would otherwise take a shared lock once per entry to discover where it is.
pub(crate) struct LevelState {
  pub(crate) id: String,
  pub(crate) unit: ProgressUnit,
  pub(crate) total: Option<u64>,
  /// Relaxed throughout: nothing is published through this counter, and a progress snapshot is not a checkpoint.
  pub(crate) completed: AtomicU64,
}

impl LevelState {
  pub(crate) fn describe(&self) -> ProgressLevel {
    ProgressLevel {
      id: self.id.clone(),
      completed: self.completed.load(Ordering::Relaxed),
      total: self.total,
      unit: self.unit,
    }
  }
}
