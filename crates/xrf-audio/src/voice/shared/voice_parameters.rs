use std::sync::atomic::{AtomicU32, Ordering};

/// A voice's continuous controls, in a block per slot that the control side writes and the audio thread reads once a
/// buffer. Unlike a command, a write never waits on the audio thread and is never refused, and writes between two
/// buffers coalesce into the last.
#[derive(Debug)]
pub(crate) struct VoiceParameters {
  /// Linear gain, as `f32` bits, zero or above.
  gain: AtomicU32,
  /// Playback speed, as `f32` bits, above zero.
  rate: AtomicU32,
}

impl VoiceParameters {
  pub(crate) fn get_gain(&self) -> f32 {
    f32::from_bits(self.gain.load(Ordering::Relaxed))
  }

  /// Sets the gain; one below zero or not a number is silence.
  pub(crate) fn set_gain(&self, gain: f32) {
    let gain: f32 = if gain.is_finite() { gain.max(0.0) } else { 0.0 };

    self.gain.store(gain.to_bits(), Ordering::Relaxed);
  }

  pub(crate) fn get_rate(&self) -> f32 {
    f32::from_bits(self.rate.load(Ordering::Relaxed))
  }

  /// Sets the playback speed; one that is not a number above zero is the source's own, since zero or less would never
  /// reach the end.
  pub(crate) fn set_rate(&self, rate: f32) {
    let rate: f32 = if rate.is_finite() && rate > 0.0 { rate } else { 1.0 };

    self.rate.store(rate.to_bits(), Ordering::Relaxed);
  }
}

impl Default for VoiceParameters {
  fn default() -> Self {
    Self {
      gain: AtomicU32::new(1f32.to_bits()),
      rate: AtomicU32::new(1f32.to_bits()),
    }
  }
}
