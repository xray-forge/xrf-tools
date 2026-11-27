use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use crate::feed::loop_frame::LoopFrame;
use crate::feed::loop_read::LoopRead;
use crate::feed::loop_status::LoopStatus;
use crate::feed::published_frame::PublishedFrame;

/// What the simulation publishes for presentation to sample: the last two frames, shared rather than copied, so a frame
/// may hold tables of any size, and whether the loop still steps. A lock is held only to clone two handles.
#[derive(Debug)]
pub struct LoopFeed<F> {
  frames: Mutex<[Option<PublishedFrame<F>>; 2]>,
  status: Mutex<LoopStatus>,
}

impl<F> Default for LoopFeed<F> {
  fn default() -> Self {
    Self {
      frames: Mutex::new([None, None]),
      status: Mutex::new(LoopStatus::Running),
    }
  }
}

impl<F> LoopFeed<F> {
  /// Publishes the frame of the tick standing at `at`, the previous one kept to read from.
  pub fn publish(&self, at: Instant, frame: LoopFrame<F>) {
    let mut frames = self.lock_frames();

    frames[0] = frames[1].take();
    frames[1] = Some(PublishedFrame {
      at,
      frame: Arc::new(frame),
    });
  }

  /// The latest frame as published, none before the first.
  pub fn get_latest(&self) -> Option<Arc<LoopFrame<F>>> {
    self.read_frames()[1]
      .as_ref()
      .map(|published| Arc::clone(&published.frame))
  }

  /// What to draw at `now`, one tick behind the latest so it always lies between two stepped frames: the previous frame
  /// at the latest's instant, the latest a tick after it. A late tick holds the latest; a single frame is drawn as it
  /// is.
  pub fn read(&self, now: Instant) -> Option<LoopRead<F>> {
    let [previous, latest] = self.read_frames();
    let latest: PublishedFrame<F> = latest?;
    let Some(previous) = previous else {
      return Some(LoopRead {
        previous: Arc::clone(&latest.frame),
        latest: latest.frame,
        alpha: 1.0,
        latest_at: latest.at,
      });
    };
    let step: Duration = latest.at.saturating_duration_since(previous.at);
    let alpha: f32 = if step.is_zero() {
      1.0
    } else {
      (now.saturating_duration_since(latest.at).as_secs_f32() / step.as_secs_f32()).min(1.0)
    };

    Some(LoopRead {
      previous: previous.frame,
      latest: latest.frame,
      alpha,
      latest_at: latest.at,
    })
  }

  /// Whether the loop still steps.
  pub fn get_status(&self) -> LoopStatus {
    self.status.lock().unwrap_or_else(PoisonError::into_inner).clone()
  }

  /// Notes that the loop no longer steps, unless a fault was noted first, which stands.
  pub(crate) fn end(&self, status: LoopStatus) {
    let mut current = self.status.lock().unwrap_or_else(PoisonError::into_inner);

    if *current == LoopStatus::Running {
      *current = status;
    }
  }

  fn read_frames(&self) -> [Option<PublishedFrame<F>>; 2] {
    self.lock_frames().clone()
  }

  fn lock_frames(&self) -> MutexGuard<'_, [Option<PublishedFrame<F>>; 2]> {
    self.frames.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
