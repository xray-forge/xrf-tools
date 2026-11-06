use std::time::Duration;

/// The engine's own clock, `Device.dwTimeGlobal`: the level's clock added up a frame at a time, each step at most
/// `LONGEST_STEP` (the engine clamps a frame's delta to 100 ms) and never back. What steps or waits by the time passed
/// (the particles, the ambient effects and the wind they bring, the camera's indoor estimate, the campfires' switching)
/// reads it, as the engine's own do, so a jump of the level's clock is one ordinary step to them: a burst's start at a
/// time of its own, or the frames' clock taking over again after it, no longer freezes them until the level's clock
/// catches up, nor steps them over minutes at once. What is a function of time (clouds, water, wind drift, object
/// motions) reads the level's clock itself, so a burst places it exactly. A level keeps one (`WorldLevel`), which its
/// effects step by and its scene's particles too (`RenderSceneFrame::engine_time`).
#[derive(Clone, Copy, Debug, Default)]
pub struct EngineClock {
  /// The level's clock at the last step, none before the first.
  last: Option<Duration>,
  now: Duration,
}

impl EngineClock {
  /// The longest step a frame takes, `fTimeDelta`'s clamp.
  pub const LONGEST_STEP: Duration = Duration::from_millis(100);

  /// Moves on to the level's clock standing at `level`, by the time passed since the last step within `LONGEST_STEP`,
  /// none where the level's clock went back; answers the engine's time then.
  pub fn advance(&mut self, level: Duration) -> Duration {
    if let Some(last) = self.last {
      self.now += level.saturating_sub(last).min(Self::LONGEST_STEP);
    }

    self.last = Some(level);

    self.now
  }
}
