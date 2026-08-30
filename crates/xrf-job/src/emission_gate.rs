use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Decides when a running job's next snapshot is worth sending, and which thread sends it.
///
/// Split from the job's own state because it is the one part of reporting that is a concurrency argument rather than a
/// description of the work: every worker of a parallel operation reaches it on every unit, and the rule it enforces —
/// at most one snapshot per interval, from whichever thread got there first — is what keeps that cheap.
pub(crate) struct EmissionGate {
  started_at: Instant,
  interval: Duration,
  /// Milliseconds since `started_at` at the last emission, and the claim one thread wins to emit.
  last_emit: AtomicU64,
}

impl EmissionGate {
  pub fn new(interval: Duration) -> Self {
    Self {
      started_at: Instant::now(),
      interval,
      last_emit: AtomicU64::new(0),
    }
  }

  /// How long the run has been going.
  pub fn elapsed(&self) -> Duration {
    self.started_at.elapsed()
  }

  /// Whether every unit is reported, which is what a test asserting the sequence asks for.
  ///
  /// Such a gate is never claimed: with no interval to enforce, a losing thread would silently drop a snapshot the
  /// test is waiting for.
  pub fn is_immediate(&self) -> bool {
    self.interval.is_zero()
  }

  /// Whether this thread is the one that reports now.
  ///
  /// The clock is read per unit rather than every so many of them. A counter in front of it would save that read, but
  /// it would also make the reporting rate a function of how many units an operation has: ten large files over three
  /// minutes would report once and then sit still until the next phase, which is precisely the frozen bar this exists
  /// to remove. The read is tens of nanoseconds against work measured in microseconds at best, and the expensive part
  /// — claiming the emission — is still reached only once an interval has actually passed.
  pub fn claim(&self) -> bool {
    let elapsed: u64 = self.elapsed_millis();
    let last: u64 = self.last_emit.load(Ordering::Relaxed);

    if elapsed.saturating_sub(last) < self.interval.as_millis() as u64 {
      return false;
    }

    // Whoever wins the exchange emits; whoever loses has nothing to add, because the winner is about to describe the
    // same stack. Losing is the common case under a pool and must stay cheap.
    self
      .last_emit
      .compare_exchange(last, elapsed, Ordering::AcqRel, Ordering::Relaxed)
      .is_ok()
  }

  /// Take the current interval without contending for it, for an emission that is never throttled.
  ///
  /// What a phase change uses: it is the highest-value thing a job says, and a reader that missed one would show the
  /// wrong phase for as long as the next one lasts.
  pub fn stamp(&self) {
    self.last_emit.store(self.elapsed_millis(), Ordering::Relaxed);
  }

  fn elapsed_millis(&self) -> u64 {
    self.started_at.elapsed().as_millis() as u64
  }
}
