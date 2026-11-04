use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::mixer::tap::audio_tap_end::AudioTapEnd;

/// What a tap's two ends share: the rate its blocks are at, the blocks the reader had no room for, and the rate the
/// mixer went on at where the stream ended for it, written by the audio thread without a lock.
#[derive(Debug, Default)]
pub(crate) struct TapShared {
  rate: AtomicU32,
  dropped: AtomicU64,
  /// Zero while the stream runs.
  ended_at_rate: AtomicU32,
}

impl TapShared {
  pub(crate) fn get_rate(&self) -> u32 {
    self.rate.load(Ordering::Acquire)
  }

  pub(crate) fn set_rate(&self, rate: u32) {
    self.rate.store(rate, Ordering::Release);
  }

  pub(crate) fn get_dropped(&self) -> u64 {
    self.dropped.load(Ordering::Relaxed)
  }

  pub(crate) fn add_dropped(&self) {
    self.dropped.fetch_add(1, Ordering::Relaxed);
  }

  /// Why the stream ended, after every block before it was pushed.
  pub(crate) fn get_end(&self) -> Option<AudioTapEnd> {
    match self.ended_at_rate.load(Ordering::Acquire) {
      0 => None,
      to => Some(AudioTapEnd::RateChanged {
        from: self.get_rate(),
        to,
      }),
    }
  }

  /// Ends the stream for the mixer going on at `rate`.
  pub(crate) fn end_at_rate(&self, rate: u32) {
    self.ended_at_rate.store(rate.max(1), Ordering::Release);
  }
}
