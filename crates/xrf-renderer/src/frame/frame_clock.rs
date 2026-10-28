use std::time::{Duration, Instant};

/// The time frames are drawn at, which everything moving, adapting or fading over them reads: the time each is drawn,
/// or one a fixed step after the last, however long it took.
///
/// A stepped clock moves alike however fast its frames are drawn, as a capture's must.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameClock {
  step: Option<Duration>,
  last: Option<Instant>,
}

impl FrameClock {
  /// Draws the frames `step` apart from the next on, or, none, at the time they are drawn.
  pub fn set_step(&mut self, step: Option<Duration>) {
    self.step = step;
  }

  /// The time the frame begun at `wall` is drawn at: `wall` itself, or its step after the last one stepped.
  pub fn tick(&mut self, wall: Instant) -> Instant {
    let now: Instant = match (self.last, self.step) {
      (Some(last), Some(step)) => last + step,
      _ => wall,
    };

    self.last = Some(now);

    now
  }
}
