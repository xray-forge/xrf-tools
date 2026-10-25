use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::job_progress::JobProgress;
use crate::progress_sink::ProgressSink;

/// Retains every snapshot instead of delivering it.
///
/// Stands in for a watcher in a test that asserts what a job reported, including how often.
#[derive(Default)]
pub struct RecordingSink {
  reported: Mutex<Vec<JobProgress>>,
}

impl RecordingSink {
  /// Everything reported so far, in the order it arrived.
  pub fn list_reported(&self) -> Vec<JobProgress> {
    self.lock().clone()
  }

  /// Everything reported so far, leaving the sink empty.
  pub fn take_reported(&self) -> Vec<JobProgress> {
    std::mem::take(&mut *self.lock())
  }

  /// A panicking reporter leaves what was already recorded readable, which is exactly what a test diagnosing the panic
  /// is trying to find out.
  fn lock(&self) -> MutexGuard<'_, Vec<JobProgress>> {
    self.reported.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl ProgressSink for RecordingSink {
  fn report(&self, progress: &JobProgress) {
    self.lock().push(progress.clone());
  }
}
