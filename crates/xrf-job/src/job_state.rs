use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::emission_gate::EmissionGate;
use crate::job_progress::JobProgress;
use crate::job_scope::JobScope;
use crate::level_state::LevelState;
use crate::progress_level::ProgressLevel;
use crate::progress_sink::ProgressSink;
use crate::progress_unit::ProgressUnit;

/// Everything one job run shares, however many threads are doing its work.
pub(crate) struct JobState {
  pub(crate) sink: Arc<dyn ProgressSink>,
  /// False for an inert job, which skips the whole emission path rather than reporting into a sink that discards.
  pub(crate) is_reporting: bool,
  /// When the run started, and which thread may report right now.
  pub(crate) gate: EmissionGate,
  pub(crate) cancelled: AtomicBool,
  /// The active stack, outermost first. Written on enter and leave, read only when a snapshot is actually being built.
  pub(crate) levels: RwLock<Vec<Arc<LevelState>>>,
  pub(crate) detail: RwLock<Option<String>>,
}

impl JobState {
  pub(crate) fn enter(state: &Arc<Self>, id: impl Into<String>, total: Option<u64>) -> JobScope {
    Self::enter_with_unit(state, id, total, ProgressUnit::Items)
  }

  pub(crate) fn enter_with_unit(
    state: &Arc<Self>,
    id: impl Into<String>,
    total: Option<u64>,
    unit: ProgressUnit,
  ) -> JobScope {
    let level: Arc<LevelState> = Arc::new(LevelState {
      id: id.into(),
      unit,
      total,
      completed: AtomicU64::new(0),
    });

    state.write_levels().push(Arc::clone(&level));

    // A phase change is the highest-value thing a job says, so it is never left to the throttle. A reader that missed
    // one would show the wrong phase for as long as the next one lasts.
    state.emit();

    JobScope::new(Arc::clone(state), level)
  }

  /// Leave a level, counting it against whatever now sits below it.
  pub(crate) fn leave(&self, level: &Arc<LevelState>) {
    let mut levels: RwLockWriteGuard<Vec<Arc<LevelState>>> = self.write_levels();

    // By identity rather than by position: scopes are dropped in the order they were entered, but a caller holding one
    // longer than the code around it suggests must remove its own level rather than somebody else's.
    if let Some(index) = levels.iter().position(|entered| Arc::ptr_eq(entered, level)) {
      levels.remove(index);
    }

    let parent: Option<Arc<LevelState>> = levels.last().cloned();

    drop(levels);

    if let Some(parent) = parent {
      parent.completed.fetch_add(1, Ordering::Relaxed);

      self.emit();
    }
  }

  /// One unit was counted somewhere; report it where the gate says this thread is the one to.
  pub(crate) fn report_if_due(&self) {
    if !self.is_reporting {
      return;
    }

    if self.gate.is_immediate() || self.gate.claim() {
      self.report();
    }
  }

  /// Emit now, regardless of the interval.
  pub(crate) fn emit(&self) {
    if !self.is_reporting {
      return;
    }

    self.gate.stamp();
    self.report();
  }

  fn report(&self) {
    let progress: JobProgress = self.describe();

    // An empty stack is a job between levels or already finished, and describing it would report a run with no phase
    // as though it had one.
    if progress.levels.is_empty() {
      return;
    }

    self.sink.report(&progress);
  }

  pub(crate) fn describe(&self) -> JobProgress {
    JobProgress {
      levels: self
        .read_levels()
        .iter()
        .map(|level| level.describe())
        .collect::<Vec<ProgressLevel>>(),
      duration: self.gate.elapsed(),
      detail: self.detail.read().unwrap_or_else(PoisonError::into_inner).clone(),
    }
  }

  /// A panicking sink or a panicking operation leaves the stack readable rather than poisoning every later snapshot:
  /// what a job had reached when it fell over is exactly what a reader is trying to find out.
  fn read_levels(&self) -> RwLockReadGuard<'_, Vec<Arc<LevelState>>> {
    self.levels.read().unwrap_or_else(PoisonError::into_inner)
  }

  fn write_levels(&self) -> RwLockWriteGuard<'_, Vec<Arc<LevelState>>> {
    self.levels.write().unwrap_or_else(PoisonError::into_inner)
  }
}
