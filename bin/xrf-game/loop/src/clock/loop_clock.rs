use std::time::{Duration, Instant};

/// When each tick stands in real time: tick `k` at `anchor + k × step`. A frame is stamped with its tick's instant
/// rather than with when it was stepped, so drawing between two frames moves evenly however late a step ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopClock {
  anchor: Instant,
  step: Duration,
}

impl LoopClock {
  /// Ticks a second unless a setting says otherwise: X-Ray's physics step, 0.01 s (`xrPhysics/Physics.cpp`), so the
  /// physics can take one step a tick.
  pub const DEFAULT_TICK_RATE: u32 = 100;

  /// A clock whose tick 0 stands at `anchor`, `tick_rate` ticks a second, at least one.
  pub fn new(anchor: Instant, tick_rate: u32) -> Self {
    Self {
      anchor,
      step: Duration::from_secs(1) / tick_rate.max(1),
    }
  }

  /// The time between two ticks.
  pub fn get_step(&self) -> Duration {
    self.step
  }

  /// When `tick` stands.
  pub fn get_instant(&self, tick: u64) -> Instant {
    let nanoseconds: u128 = self.step.as_nanos() * u128::from(tick);

    self.anchor + Duration::from_nanos(u64::try_from(nanoseconds).unwrap_or(u64::MAX))
  }

  /// The last tick standing at or before `now`, 0 before the anchor.
  pub fn get_tick_at(&self, now: Instant) -> u64 {
    let elapsed: u128 = now.saturating_duration_since(self.anchor).as_nanos();

    u64::try_from(elapsed / self.step.as_nanos().max(1)).unwrap_or(u64::MAX)
  }

  /// Moves the anchor so that `tick` stands at `now`: the clock gives up the time it fell behind rather than stepping
  /// through it all at once.
  pub fn slip(&mut self, tick: u64, now: Instant) {
    let behind: Duration = self.get_instant(tick).saturating_duration_since(self.anchor);

    self.anchor = now.checked_sub(behind).unwrap_or(now);
  }
}
