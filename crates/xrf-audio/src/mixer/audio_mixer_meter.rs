use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::mixer::audio_mixer_load::AudioMixerLoad;

/// The mixer's clock and load, written by the audio thread once a buffer and read without a lock from any thread.
#[derive(Clone, Debug, Default)]
pub struct AudioMixerMeter {
  shared: Arc<MeterShared>,
}

#[derive(Debug, Default)]
struct MeterShared {
  frames: AtomicU64,
  load: AtomicU32,
  peak_load: AtomicU32,
  overruns: AtomicU64,
  voices: AtomicU32,
}

impl AudioMixerMeter {
  /// Frames the mixer has rendered.
  pub fn get_frames(&self) -> u64 {
    self.shared.frames.load(Ordering::Acquire)
  }

  /// The mixer's load, starting the peak over.
  pub fn read(&self) -> AudioMixerLoad {
    let shared: &MeterShared = &self.shared;

    AudioMixerLoad {
      frames: shared.frames.load(Ordering::Acquire),
      load: f32::from_bits(shared.load.load(Ordering::Relaxed)),
      peak_load: f32::from_bits(shared.peak_load.swap(0, Ordering::Relaxed)),
      overruns: shared.overruns.load(Ordering::Relaxed),
      voices: shared.voices.load(Ordering::Relaxed),
    }
  }

  /// Notes a buffer of `frames` that took `load` of the time it plays for, with `voices` held after it.
  pub(crate) fn record(&self, frames: u64, load: f32, voices: u32) {
    let shared: &MeterShared = &self.shared;
    let load: f32 = if load.is_finite() { load.max(0.0) } else { 0.0 };

    shared.load.store(load.to_bits(), Ordering::Relaxed);
    // A non-negative float's bits order as the float does.
    shared.peak_load.fetch_max(load.to_bits(), Ordering::Relaxed);
    shared.voices.store(voices, Ordering::Relaxed);

    if load > 1.0 {
      shared.overruns.fetch_add(1, Ordering::Relaxed);
    }

    shared.frames.fetch_add(frames, Ordering::Release);
  }
}
