use std::cmp::max;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// The clock and the counter one unpacking run reports itself with.
///
/// Held apart from the run because timing the two phases and deciding how often to log is a story of its own, and it
/// had already drifted once when two unpack paths each kept their own copy of it.
pub(crate) struct ArchiveUnpackProgress {
  started_at: Instant,
  /// How long creating the destination tree took, which precedes any payload being written.
  prepared_at: Duration,
  /// Shared across the pool: a run driven by workers has no join point left to count entries at.
  completed: AtomicUsize,
  total: usize,
  /// Entries between progress lines, so a large set logs about twenty times and a small one still says something.
  step: usize,
}

impl ArchiveUnpackProgress {
  /// Starts the clock, before the destination tree is prepared.
  pub(crate) fn begin(total: usize) -> Self {
    Self {
      started_at: Instant::now(),
      prepared_at: Duration::ZERO,
      completed: AtomicUsize::new(0),
      total,
      step: max(total / 100 * 5, 5),
    }
  }

  /// Closes the preparation phase, once the destination tree exists.
  pub(crate) fn record_prepared(&mut self) {
    self.prepared_at = self.started_at.elapsed();
  }

  /// Counts one entry as dealt with, whether it was written or had nothing to write.
  ///
  /// Every worker calls this, so the count is atomic and the line is emitted by whichever worker happens to land on a
  /// step. Relaxed ordering is enough: nothing is published through this counter, and a log line is not a checkpoint.
  pub(crate) fn record_unpacked(&self) {
    let completed: usize = self.completed.fetch_add(1, Ordering::Relaxed) + 1;

    if completed.is_multiple_of(self.step) {
      log::info!("Unpacked {}/{} files", completed, self.total);
    }
  }

  /// How long preparing the destination tree took.
  pub(crate) fn get_prepared_at(&self) -> Duration {
    self.prepared_at
  }

  /// How long the run has taken so far, preparation included.
  pub(crate) fn elapsed(&self) -> Duration {
    self.started_at.elapsed()
  }
}
