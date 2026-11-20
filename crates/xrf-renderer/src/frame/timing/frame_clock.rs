use std::time::{Duration, Instant};

/// The time frames are drawn at: the wall time, or a fixed step after the last frame however long it took.
///
/// A stepped clock advances the same at any frame rate, as a capture needs. Frames drawn faster than their step run it
/// ahead of the wall, and it keeps that lead once the step is cleared rather than jumping back.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FrameClock {
  step: Option<Duration>,
  last: Option<Instant>,
  /// How far the last frame stood ahead of the wall; an unstepped clock keeps it.
  lead: Duration,
}

impl FrameClock {
  /// Draws the frames `step` apart from the next on, or, none, at the time they are drawn.
  pub(crate) fn set_step(&mut self, step: Option<Duration>) {
    self.step = step;
  }

  /// The draw time of the frame begun at `wall`: the last plus the step when stepped (held on overflow), else `wall`
  /// plus the kept lead. Never before the last.
  pub(crate) fn tick(&mut self, wall: Instant) -> Instant {
    let now: Instant = match (self.last, self.step) {
      (Some(last), Some(step)) => last.checked_add(step).unwrap_or(last),
      _ => wall.checked_add(self.lead).unwrap_or(wall),
    };
    let now: Instant = self.last.map_or(now, |last| now.max(last));

    self.lead = now.saturating_duration_since(wall);
    self.last = Some(now);

    now
  }
}
