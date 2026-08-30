use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::job_progress::JobProgress;

/// Delivers a job's progress snapshots to whoever is watching.
///
/// One method, so a new sink is written once and cannot answer some snapshots while silently dropping others. Coalescing
/// is not a sink's concern: by the time a snapshot arrives here the handle has already decided it is worth sending, and
/// a sink that throttled again would make the rate a function of which sink happened to be attached.
///
/// Called from whichever thread tripped the emission gate, which for a parallel operation is an arbitrary worker. An
/// implementation that blocks blocks that worker.
pub trait ProgressSink: Send + Sync {
  fn report(&self, progress: &JobProgress);
}

/// Discards every snapshot.
///
/// What a job reporting to nobody holds, so an operation never branches on whether anyone is watching.
#[derive(Default)]
pub struct NoopSink;

impl ProgressSink for NoopSink {
  fn report(&self, _: &JobProgress) {}
}

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
