use std::time::Duration;

/// A time on a level's clock, how long it has lived on the frames' clock, kept whole. Nothing reads it as `f32` seconds,
/// which after a day of uptime step by 7.8 ms, more than a frame at 160 Hz: an animation takes its own periodic phase of
/// it, worked out in `f64`, and a step between frames is a difference of two whole times.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LevelTime(Duration);

impl LevelTime {
  /// The level's clock `elapsed` after it began opening.
  pub const fn new(elapsed: Duration) -> Self {
    Self(elapsed)
  }

  /// The same time whole.
  pub fn get_duration(self) -> Duration {
    self.0
  }

  /// The same time as seconds, at `f64`'s precision.
  pub fn get_seconds(self) -> f64 {
    self.0.as_secs_f64()
  }

  /// Seconds into the repeat of `period` seconds it stands in, from nought to the period; nought for no period.
  pub fn get_phase(self, period: f64) -> f32 {
    self.get_cycle(1.0, period)
  }

  /// Turns of something turning `rate` times a second, from nought to one.
  pub fn get_turns(self, rate: f64) -> f32 {
    self.get_cycle(rate, 1.0)
  }

  /// How far something moving `rate` a second has come, taken within its repeat of `period`: the same as the product
  /// wherever the reader repeats over `period`, as fine at any uptime. Nought for no period or a product past `f64`.
  pub fn get_cycle(self, rate: f64, period: f64) -> f32 {
    Self::wrap(self.get_seconds() * rate, period)
  }

  /// Seconds since `earlier`, nought where it is not earlier.
  pub fn get_since(self, earlier: Self) -> f32 {
    self.0.saturating_sub(earlier.0).as_secs_f32()
  }

  /// `value` taken within its repeat of `period`, from nought to the period, as `f32`; nought for no period or a value
  /// that is not finite. For a sum kept in `f64` rather than a time.
  pub fn wrap(value: f64, period: f64) -> f32 {
    if !(period > 0.0 && value.is_finite()) {
      return 0.0;
    }

    // `rem_euclid` can round up to the period itself for a value a hair under a multiple of it.
    let wrapped: f64 = value.rem_euclid(period);

    if wrapped >= period { 0.0 } else { wrapped as f32 }
  }
}
