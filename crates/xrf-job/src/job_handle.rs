use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};

use crate::job_progress::JobProgress;
use crate::job_scope::{JobScope, LevelState};
use crate::progress_level::ProgressLevel;
use crate::progress_sink::{NoopSink, ProgressSink};
use crate::progress_unit::ProgressUnit;

/// How often a running job reports, at most.
///
/// Time rather than a count of units, because units are not the same size: a step of one in twenty reports twenty times
/// over three seconds for a set of small configs and ten times over several minutes for a set of level meshes, and
/// neither is what a person watching needs. Ten a second is more than a reader can follow and few enough that the
/// transport underneath never becomes the cost of doing the work.
pub const DEFAULT_PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

/// Everything one job run shares, however many threads are doing its work.
pub(crate) struct JobState {
  started_at: Instant,
  sink: Arc<dyn ProgressSink>,
  /// False for an inert job, which skips the whole emission path rather than reporting into a sink that discards.
  is_reporting: bool,
  interval: Duration,
  cancelled: AtomicBool,
  /// The active stack, outermost first. Written on enter and leave, read only when a snapshot is actually being built.
  levels: RwLock<Vec<Arc<LevelState>>>,
  detail: RwLock<Option<String>>,
  /// Milliseconds since `started_at` at the last emission, and the claim one thread wins to emit.
  last_emit: AtomicU64,
}

/// A job's progress and its cancellation, as one thing an operation is handed.
///
/// One object rather than two because they are one design: a run that can report how far it has got is a run somebody
/// can decide to stop, and an operation that took a reporter and a token separately would eventually be given one
/// without the other. Cloning is cheap and shares the same run — the app clones one into a blocking thread while
/// keeping the original to cancel with.
#[derive(Clone)]
pub struct JobHandle {
  state: Arc<JobState>,
}

impl JobHandle {
  /// A job reporting to `sink`, at the default interval.
  pub fn new(sink: Arc<dyn ProgressSink>) -> Self {
    Self::with_interval(sink, DEFAULT_PROGRESS_INTERVAL)
  }

  /// A job reporting to `sink` no more often than `interval`.
  ///
  /// A zero interval reports every unit, which is what a test asserting the sequence wants and what nothing in
  /// production should ask for.
  pub fn with_interval(sink: Arc<dyn ProgressSink>, interval: Duration) -> Self {
    Self {
      state: Arc::new(JobState {
        started_at: Instant::now(),
        sink,
        is_reporting: true,
        interval,
        cancelled: AtomicBool::new(false),
        levels: RwLock::new(Vec::new()),
        detail: RwLock::new(None),
        last_emit: AtomicU64::new(0),
      }),
    }
  }

  /// A job that reports nowhere and is never cancelled.
  ///
  /// What the command line and every test that is not about progress pass, so an operation has one signature rather
  /// than an instrumented path and a bare one.
  pub fn inert() -> Self {
    Self {
      state: Arc::new(JobState {
        started_at: Instant::now(),
        sink: Arc::new(NoopSink),
        is_reporting: false,
        interval: DEFAULT_PROGRESS_INTERVAL,
        cancelled: AtomicBool::new(false),
        levels: RwLock::new(Vec::new()),
        detail: RwLock::new(None),
        last_emit: AtomicU64::new(0),
      }),
    }
  }

  /// Enter a level, reporting it until the returned scope is dropped.
  ///
  /// `total` is what this level is made of — its child levels where it has them, its own units where it does not — and
  /// `None` where that cannot be known before the work is done.
  pub fn enter(&self, id: impl Into<String>, total: Option<u64>) -> JobScope {
    JobState::enter(&self.state, id, total)
  }

  /// Enter a level whose counts are bytes rather than things.
  pub fn enter_bytes(&self, id: impl Into<String>, total: Option<u64>) -> JobScope {
    JobState::enter_with_unit(&self.state, id, total, ProgressUnit::Bytes)
  }

  /// Name what the job is on right now, or clear it.
  ///
  /// For sequential work only. An operation spreading its units over a pool leaves this alone: the entry one arbitrary
  /// worker happens to be on flickers between unrelated files, which reads as thrashing rather than as progress.
  pub fn set_detail(&self, detail: Option<String>) {
    *self.state.detail.write().unwrap_or_else(PoisonError::into_inner) = detail;
  }

  /// Ask the job to stop at its next safe boundary.
  ///
  /// Relaxed, because nothing is published through this flag — the operation reads it, decides to stop, and reports
  /// what it had already finished through its own return value.
  pub fn cancel(&self) {
    self.state.cancelled.store(true, Ordering::Relaxed);
  }

  /// Whether stopping has been asked for.
  ///
  /// The form to use before starting expensive work, which is the only place a check helps: a write already begun
  /// cannot be halved, so the boundary has to be found before it rather than inside it.
  pub fn is_cancelled(&self) -> bool {
    self.state.cancelled.load(Ordering::Relaxed)
  }

  /// The same question as an error, for a loop that stops by returning one.
  ///
  /// Exists because a parallel iterator has no other early exit: `try_for_each` stops on `Err` and on nothing else. The
  /// error is control flow, and an operation catches its own rather than letting it reach a caller — a caller told only
  /// that the run stopped cannot say which files are now on disk.
  pub fn check_cancelled(&self) -> XrfResult<()> {
    if self.is_cancelled() {
      return Err(XrfError::new_cancelled_error("job was cancelled"));
    }

    Ok(())
  }

  /// How long the run has taken so far.
  pub fn elapsed(&self) -> Duration {
    self.state.started_at.elapsed()
  }

  /// What a snapshot would say right now, whether or not one is due.
  ///
  /// For tests and for a caller that wants the state without waiting for the interval; the running operation never
  /// needs it.
  pub fn snapshot(&self) -> JobProgress {
    self.state.describe()
  }
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
      label: None,
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

  /// One unit was counted somewhere; decide whether that is worth saying.
  ///
  /// The clock is read per unit rather than every so many of them. A counter in front of it would save that read, but
  /// it would also make the reporting rate a function of how many units an operation has: ten large files over three
  /// minutes would report once and then sit still until the next phase, which is precisely the frozen bar this exists
  /// to remove. The read is tens of nanoseconds against work measured in microseconds at best, and the expensive part
  /// — claiming the emission — is still reached only once an interval has actually passed.
  pub(crate) fn on_unit(&self) {
    if !self.is_reporting {
      return;
    }

    // A zero interval says everything, which is what a test asserting the sequence asks for. It skips the claim too:
    // with no interval to enforce, a losing thread would silently drop a snapshot the test is waiting for.
    if self.interval.is_zero() {
      return self.report();
    }

    self.emit_if_due();
  }

  /// Emit if the interval has passed and this thread is the one that claims it.
  fn emit_if_due(&self) {
    let elapsed: u64 = self.started_at.elapsed().as_millis() as u64;
    let last: u64 = self.last_emit.load(Ordering::Relaxed);

    if elapsed.saturating_sub(last) < self.interval.as_millis() as u64 {
      return;
    }

    // Whoever wins the exchange emits; whoever loses has nothing to add, because the winner is about to describe the
    // same stack. Losing is the common case under a pool and must stay cheap.
    if self
      .last_emit
      .compare_exchange(last, elapsed, Ordering::AcqRel, Ordering::Relaxed)
      .is_err()
    {
      return;
    }

    self.report();
  }

  /// Emit now, regardless of the interval.
  pub(crate) fn emit(&self) {
    if !self.is_reporting {
      return;
    }

    self
      .last_emit
      .store(self.started_at.elapsed().as_millis() as u64, Ordering::Relaxed);

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

  fn describe(&self) -> JobProgress {
    JobProgress {
      levels: self
        .read_levels()
        .iter()
        .map(|level| level.describe())
        .collect::<Vec<ProgressLevel>>(),
      duration: self.started_at.elapsed(),
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
