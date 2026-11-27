use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use crate::feed::loop_frame::LoopFrame;
use crate::feed::published_frame::PublishedFrame;
use crate::state::loop_blend::LoopBlend;

/// The last two frames the simulation published, which presentation reads between: a lock held only to copy them, so
/// neither side waits on the other's work.
#[derive(Debug)]
pub struct LoopFeed<F> {
  frames: Mutex<[Option<PublishedFrame<F>>; 2]>,
}

impl<F> Default for LoopFeed<F> {
  fn default() -> Self {
    Self {
      frames: Mutex::new([None, None]),
    }
  }
}

impl<F: LoopBlend> LoopFeed<F> {
  /// Publishes the frame of the tick standing at `at`, the previous one kept to read from.
  pub fn publish(&self, at: Instant, frame: LoopFrame<F>) {
    let mut frames = self.frames.lock().unwrap_or_else(PoisonError::into_inner);

    frames[0] = frames[1].take();
    frames[1] = Some(PublishedFrame { at, frame });
  }

  /// The latest frame as published, none before the first.
  pub fn get_latest(&self) -> Option<LoopFrame<F>> {
    self.read_frames()[1].map(|published| published.frame)
  }

  /// The state to draw at `now`, one tick behind the latest so it always lies between two stepped frames: the previous
  /// frame at the latest's instant, the latest a tick after it. A late tick holds the latest; a single frame is drawn
  /// as it is.
  pub fn read(&self, now: Instant) -> Option<LoopFrame<F>> {
    let [previous, latest] = self.read_frames();
    let latest: PublishedFrame<F> = latest?;
    let Some(previous) = previous else {
      return Some(latest.frame);
    };
    let step: Duration = latest.at.saturating_duration_since(previous.at);

    if step.is_zero() {
      return Some(latest.frame);
    }

    let alpha: f32 = (now.saturating_duration_since(latest.at).as_secs_f32() / step.as_secs_f32()).min(1.0);

    Some(previous.frame.blend(&latest.frame, alpha))
  }

  fn read_frames(&self) -> [Option<PublishedFrame<F>>; 2] {
    *self.frames.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
