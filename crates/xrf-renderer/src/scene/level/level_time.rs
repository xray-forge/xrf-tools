use std::time::Duration;

/// Time since the level began opening, kept as a whole `Duration`.
///
/// Never read it as `f32` seconds, which step by 7.8 ms after a day of uptime: take a periodic phase in `f64`
/// ([`Self::get_cycle`]) or a difference of two times.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct LevelTime(Duration);

impl LevelTime {
  pub(crate) const fn new(elapsed: Duration) -> Self {
    Self(elapsed)
  }

  pub(crate) fn get_seconds(self) -> f64 {
    self.0.as_secs_f64()
  }

  /// Seconds into the current repeat of `period`, in `[0, period)`; zero for a non-positive period.
  pub(crate) fn get_phase(self, period: f64) -> f32 {
    self.get_cycle(1.0, period)
  }

  /// Fraction of a turn, in `[0, 1)`, for something turning `rate` times a second.
  pub(crate) fn get_turns(self, rate: f64) -> f32 {
    self.get_cycle(rate, 1.0)
  }

  /// Seconds times `rate`, wrapped into `[0, period)`, precise at any uptime for a reader that repeats over `period`.
  /// Zero for a non-positive period or a non-finite product.
  pub(crate) fn get_cycle(self, rate: f64, period: f64) -> f32 {
    Self::wrap(self.get_seconds() * rate, period)
  }

  /// Seconds since `earlier`; zero when `earlier` is not earlier.
  pub(crate) fn get_since(self, earlier: Self) -> f32 {
    self.0.saturating_sub(earlier.0).as_secs_f32()
  }

  /// Wraps an `f64` sum into `[0, period)` as `f32`; zero for a non-positive period or a non-finite value.
  pub(crate) fn wrap(value: f64, period: f64) -> f32 {
    if !(period > 0.0 && value.is_finite()) {
      return 0.0;
    }

    // `rem_euclid` can round up to the period itself for a value a hair under a multiple of it.
    let wrapped: f64 = value.rem_euclid(period);

    if wrapped >= period { 0.0 } else { wrapped as f32 }
  }
}
