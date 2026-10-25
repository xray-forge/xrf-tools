use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

use crate::job::EmissionGate;
use crate::job::JobScope;
use crate::job::LevelState;
use crate::progress::JobProgress;
use crate::progress::ProgressLevel;
use crate::progress::ProgressSink;
use crate::progress::ProgressUnit;

/// Everything one job run shares, however many threads are doing its work.
pub(crate) struct JobState {
  sink: Arc<dyn ProgressSink>,
  /// False for an inert job, which skips the whole emission path rather than reporting into a sink that discards.
  is_reporting: bool,
  /// When the run started, and which thread may report right now.
  gate: EmissionGate,
  /// Relaxed, because nothing is published through this flag: the operation reads it, decides to stop, and reports what
  /// it had already finished through its own return value.
  cancelled: AtomicBool,
  /// The active stack, outermost first. Written on enter and leave, read only when a snapshot is actually being built.
  levels: RwLock<Vec<Arc<LevelState>>>,
  detail: RwLock<Option<String>>,
}

impl JobState {
  pub(crate) fn new(sink: Arc<dyn ProgressSink>, is_reporting: bool, interval: Duration) -> Self {
    Self {
      sink,
      is_reporting,
      gate: EmissionGate::new(interval),
      cancelled: AtomicBool::new(false),
      levels: RwLock::new(Vec::new()),
      detail: RwLock::new(None),
    }
  }

  pub(crate) fn enter(state: &Arc<Self>, id: impl Into<String>, total: Option<u64>) -> JobScope {
    Self::enter_with_unit(state, id, total, ProgressUnit::Items)
  }

  pub(crate) fn enter_with_unit(
    state: &Arc<Self>,
    id: impl Into<String>,
    total: Option<u64>,
    unit: ProgressUnit,
  ) -> JobScope {
    let level: Arc<LevelState> = Arc::new(LevelState::new(id.into(), unit, total));

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
      parent.advance_by(1);

      self.emit();
    }
  }

  pub(crate) fn set_detail(&self, detail: Option<String>) {
    *self.detail.write().unwrap_or_else(PoisonError::into_inner) = detail;
  }

  pub(crate) fn cancel(&self) {
    self.cancelled.store(true, Ordering::Relaxed);
  }

  pub(crate) fn is_cancelled(&self) -> bool {
    self.cancelled.load(Ordering::Relaxed)
  }

  pub(crate) fn elapsed(&self) -> Duration {
    self.gate.elapsed()
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
