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
  pub fn with_interval(sink: Arc<dyn ProgressSink>, interval: Duration) -> Self {
    Self {
      state: Arc::new(JobState::new(sink, true, interval)),
    }
  }

  /// A job that reports nowhere, though whoever holds it can still cancel it.
  pub fn inert() -> Self {
    Self {
      state: Arc::new(JobState::new(
        Arc::new(NoopSink),
        false,
        Self::DEFAULT_PROGRESS_INTERVAL,
      )),
    }
  }

  /// The same run, reporting nothing: cancelling either handle cancels both and their clocks agree, but levels, units
  /// and detail given to this one reach no sink.
  pub fn without_progress(&self) -> Self {
    Self {
      state: Arc::new(self.state.without_progress()),
    }
  }

  /// Enter a level, reporting it until the returned scope is dropped.
  pub fn enter(&self, id: impl Into<String>, total: Option<u64>) -> JobScope {
    JobState::enter(&self.state, id, total)
  }

  /// Enter a level whose counts are bytes rather than things.
  pub fn enter_bytes(&self, id: impl Into<String>, total: Option<u64>) -> JobScope {
    JobState::enter_with_unit(&self.state, id, total, ProgressUnit::Bytes)
  }

  /// Name what the job is on right now, or clear it.
  pub fn set_detail(&self, detail: Option<String>) {
    self.state.set_detail(detail);
  }

  /// Ask the job to stop at its next safe boundary.
  pub fn cancel(&self) {
    self.state.cancel();
  }

  /// Whether stopping has been asked for.
  pub fn is_cancelled(&self) -> bool {
    self.state.is_cancelled()
  }

  /// The same question as an error, for a loop that stops by returning one.
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
  pub fn report_if_due(&self) {
    self.state.report_if_due();
  }

  /// What a watcher would be told right now, whether or not an emission is due.
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
