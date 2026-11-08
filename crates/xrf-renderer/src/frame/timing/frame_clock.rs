use std::time::{Duration, Instant};

/// The time frames are drawn at, which everything moving, adapting or fading over them reads: the time each is drawn,
/// or one a fixed step after the last, however long it took.
///
/// A stepped clock moves alike however fast its frames are drawn, as a capture's must. Frames drawn faster than their
/// step run it ahead of the wall's time, and it keeps that lead once the step is cleared, following the wall from where
/// it stood rather than going back to it.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameClock {
  step: Option<Duration>,
  last: Option<Instant>,
  /// How far ahead of the wall's time its last frame stood, which an unstepped clock keeps.
  lead: Duration,
}

impl FrameClock {
  /// Draws the frames `step` apart from the next on, or, none, at the time they are drawn.
  pub fn set_step(&mut self, step: Option<Duration>) {
    self.step = step;
  }

  /// The time the frame begun at `wall` is drawn at: its step after the last one stepped, held at the last where that
  /// step would pass the latest time the platform's clock can stand for; else `wall` plus the lead a stepped clock left.
  /// Never before the last.
  pub fn tick(&mut self, wall: Instant) -> Instant {
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
