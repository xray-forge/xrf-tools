use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Decides when a running job's next snapshot is worth sending, and which thread sends it.
pub(crate) struct EmissionGate {
  started_at: Instant,
  interval: Duration,
  /// Milliseconds since `started_at` at the last emission, and the claim one thread wins to emit.
  last_emit: AtomicU64,
}

impl EmissionGate {
  pub(crate) fn new(interval: Duration) -> Self {
    Self {
      started_at: Instant::now(),
      interval,
      last_emit: AtomicU64::new(0),
    }
  }

  /// A gate on the same clock that has never emitted.
  pub(crate) fn sharing_clock(&self) -> Self {
    Self {
      started_at: self.started_at,
      interval: self.interval,
      last_emit: AtomicU64::new(0),
    }
  }

  /// How long the run has been going.
  pub(crate) fn elapsed(&self) -> Duration {
    self.started_at.elapsed()
  }

  /// Whether every unit is reported, which is what a test asserting the sequence asks for.
  pub(crate) fn is_immediate(&self) -> bool {
    self.interval.is_zero()
  }

  /// Whether this thread is the one that reports now.
  pub(crate) fn claim(&self) -> bool {
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
  pub(crate) fn stamp(&self) {
    self.last_emit.store(self.elapsed_millis(), Ordering::Relaxed);
  }

  fn elapsed_millis(&self) -> u64 {
    self.started_at.elapsed().as_millis() as u64
  }
}
