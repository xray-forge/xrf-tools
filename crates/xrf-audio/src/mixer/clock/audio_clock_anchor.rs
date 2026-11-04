use std::time::{Duration, Instant};

/// Where the mixer's frames play: frame `frame` was handed to the device at `at`, and plays `latency` after, so every
/// frame plays a frame's length after the one before it. Video's present times go on the same line, so a recording
/// places the two as they were seen and heard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioClockAnchor {
  pub frame: u64,
  pub at: Instant,
  pub latency: Duration,
}

impl AudioClockAnchor {
  /// When the mixer's frame `frame` plays, at `rate` frames a second.
  pub fn get_instant(&self, frame: u64, rate: u32) -> Instant {
    let plays: Instant = self.at + self.latency;
    let rate: f64 = f64::from(rate.max(1));

    if frame >= self.frame {
      plays + Duration::from_secs_f64((frame - self.frame) as f64 / rate)
    } else {
      plays
        .checked_sub(Duration::from_secs_f64((self.frame - frame) as f64 / rate))
        .unwrap_or(plays)
    }
  }

  /// The mixer's frame that plays at `instant`, at `rate` frames a second, the first frame for an instant before it.
  pub fn get_frame(&self, instant: Instant, rate: u32) -> u64 {
    let plays: Instant = self.at + self.latency;
    let rate: f64 = f64::from(rate.max(1));

    if instant >= plays {
      self.frame + (instant.duration_since(plays).as_secs_f64() * rate).round() as u64
    } else {
      self
        .frame
        .saturating_sub((plays.duration_since(instant).as_secs_f64() * rate).round() as u64)
    }
  }
}
