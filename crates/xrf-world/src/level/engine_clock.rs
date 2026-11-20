use std::time::Duration;

/// The engine's own clock, `Device.dwTimeGlobal`: the level's clock added up a frame at a time, each step at most
/// `LONGEST_STEP` and never back.
///
/// What steps or waits by the time passed (particles, ambient effects and their wind, the camera's indoor estimate,
/// campfires) reads it, so a jump of the level's clock, such as a burst's start, is one ordinary step to them. What is
/// a function of time (clouds, water, wind drift, object motions) reads the level's clock, which a burst sets exactly.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EngineClock {
  /// The level's clock at the last step, none before the first.
  last: Option<Duration>,
  now: Duration,
}

impl EngineClock {
  /// The longest step a frame takes, `fTimeDelta`'s clamp.
  pub(crate) const LONGEST_STEP: Duration = Duration::from_millis(100);

  /// Steps to the level's clock at `level`, by the time passed within `LONGEST_STEP` and none where it went back, and
  /// returns the engine's time.
  pub(crate) fn advance(&mut self, level: Duration) -> Duration {
    if let Some(last) = self.last {
      self.now += level.saturating_sub(last).min(Self::LONGEST_STEP);
    }

    self.last = Some(level);

    self.now
  }
}
