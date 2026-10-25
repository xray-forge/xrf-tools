use std::sync::Arc;
use std::time::Duration;

use xrf_error::{XrfError, XrfResult};

use crate::job::JobScope;
use crate::job::JobState;
use crate::progress::JobProgress;
use crate::progress::NoopSink;
use crate::progress::ProgressSink;
use crate::progress::ProgressUnit;

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
  /// How often a running job reports, at most.
  pub const DEFAULT_PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

  /// A job reporting to `sink`, at the default interval.
  pub fn new(sink: Arc<dyn ProgressSink>) -> Self {
    Self::with_interval(sink, Self::DEFAULT_PROGRESS_INTERVAL)
  }

  /// A job reporting to `sink` no more often than `interval`.
  ///
  /// A zero interval reports every unit, which is what a test asserting the sequence wants and what nothing in
  /// production should ask for.
  pub fn with_interval(sink: Arc<dyn ProgressSink>, interval: Duration) -> Self {
    Self {
      state: Arc::new(JobState::new(sink, true, interval)),
    }
  }

  /// A job that reports nowhere, though whoever holds it can still cancel it.
  ///
  /// What the command line and every test that is not about progress pass, so an operation has one signature rather
  /// than an instrumented path and a bare one.
  pub fn inert() -> Self {
    Self {
      state: Arc::new(JobState::new(
        Arc::new(NoopSink),
        false,
        Self::DEFAULT_PROGRESS_INTERVAL,
      )),
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
    self.state.set_detail(detail);
  }

  /// Ask the job to stop at its next safe boundary.
  pub fn cancel(&self) {
    self.state.cancel();
  }

  /// Whether stopping has been asked for.
  ///
  /// The form to use before starting expensive work, which is the only place a check helps: a write already begun
  /// cannot be halved, so the boundary has to be found before it rather than inside it.
  pub fn is_cancelled(&self) -> bool {
    self.state.is_cancelled()
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
    self.state.elapsed()
  }

  /// Says where this job has got to, if it has not said so recently.
  ///
  /// The same rule a completed unit goes through, for a caller that has no unit to report: a phase doing real work
  /// with nothing countable in it — mounting roots, indexing a tree, assembling a project — otherwise says nothing
  /// for as long as it lasts, and a window watching it shows an elapsed time that never moves.
  ///
  /// Asking rather than telling, because when a job last spoke is the job's own business. A caller that sent a
  /// snapshot itself would double what a reporting phase says and would have to keep its own idea of the interval.
  pub fn report_if_due(&self) {
    self.state.report_if_due();
  }

  /// What a watcher would be told right now, whether or not an emission is due.
  ///
  /// `None` while the job is between levels, which is the same thing the sink is spared: a snapshot with no levels
  /// describes a run with no phase as though it had one. Lets a listing read a running job's state on demand rather
  /// than the job having to push every snapshot somewhere a listing can find it.
  pub fn get_progress(&self) -> Option<JobProgress> {
    let progress: JobProgress = self.state.describe();

    (!progress.levels.is_empty()).then_some(progress)
  }
}

impl Default for JobHandle {
  /// A job that reports nowhere, so a caller assembling options without one is not silently given a watcher.
  fn default() -> Self {
    Self::inert()
  }
}
