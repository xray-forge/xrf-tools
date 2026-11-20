use std::sync::atomic::{AtomicI64, AtomicU32, AtomicU64, Ordering, fence};
use std::time::{Duration, Instant};

use crate::mixer::clock::audio_clock_anchor::AudioClockAnchor;

/// An [`AudioClockAnchor`] written by whoever renders the mixer and read from any thread, without a lock or an
/// allocation: a sequence lock, odd while a write is under way, whose reader retries a read a write overlapped.
#[derive(Debug)]
pub(crate) struct ClockAnchorCell {
  /// What the anchor's instants are kept as nanoseconds from, either side: one rendered with no device is anchored
  /// before the moment it renders.
  base: Instant,
  /// Twice the writes so far, plus one during a write; zero before the first.
  sequence: AtomicU32,
  frame: AtomicU64,
  at: AtomicI64,
  latency: AtomicU64,
}

impl Default for ClockAnchorCell {
  fn default() -> Self {
    Self {
      base: Instant::now(),
      sequence: AtomicU32::new(0),
      frame: AtomicU64::new(0),
      at: AtomicI64::new(0),
      latency: AtomicU64::new(0),
    }
  }
}

impl ClockAnchorCell {
  /// The anchor last written, none before the first. One writer at a time: the mixer is rendered by one thread at once.
  pub(crate) fn read(&self) -> Option<AudioClockAnchor> {
    loop {
      let before: u32 = self.sequence.load(Ordering::Acquire);

      if before == 0 {
        return None;
      }

      if before % 2 == 1 {
        std::hint::spin_loop();
        continue;
      }

      let frame: u64 = self.frame.load(Ordering::Relaxed);
      let at: i64 = self.at.load(Ordering::Relaxed);
      let latency: u64 = self.latency.load(Ordering::Relaxed);

      fence(Ordering::Acquire);

      if self.sequence.load(Ordering::Relaxed) == before {
        let offset: Duration = Duration::from_nanos(at.unsigned_abs());

        return Some(AudioClockAnchor {
          frame,
          at: if at >= 0 {
            self.base + offset
          } else {
            self.base.checked_sub(offset).unwrap_or(self.base)
          },
          latency: Duration::from_nanos(latency),
        });
      }
    }
  }

  pub(crate) fn write(&self, anchor: AudioClockAnchor) {
    let at: i64 = if anchor.at >= self.base {
      anchor.at.duration_since(self.base).as_nanos() as i64
    } else {
      -(self.base.duration_since(anchor.at).as_nanos() as i64)
    };

    self.sequence.fetch_add(1, Ordering::Relaxed);
    fence(Ordering::Release);
    self.frame.store(anchor.frame, Ordering::Relaxed);
    self.at.store(at, Ordering::Relaxed);
    self.latency.store(anchor.latency.as_nanos() as u64, Ordering::Relaxed);
    self.sequence.fetch_add(1, Ordering::Release);
  }
}
