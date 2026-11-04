use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::mixer::audio_mixer_load::AudioMixerLoad;
use crate::mixer::clock::audio_clock_anchor::AudioClockAnchor;
use crate::mixer::clock::clock_anchor_cell::ClockAnchorCell;

/// The mixer's clock and load, and where its frames play, written by the audio thread once a buffer and read without a
/// lock from any thread.
#[derive(Clone, Debug, Default)]
pub struct AudioMixerMeter {
  shared: Arc<MeterShared>,
}

#[derive(Debug, Default)]
struct MeterShared {
  /// Where the frames play: each device buffer's start, or the device's pace while none plays.
  anchor: ClockAnchorCell,
  frames: AtomicU64,
  /// The rate it renders at, frames a second.
  rate: AtomicU32,
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

  /// The last buffer's share of the time it plays for, leaving the peak as it is.
  pub fn get_load(&self) -> f32 {
    f32::from_bits(self.shared.load.load(Ordering::Relaxed))
  }

  /// The rate the mixer renders at, frames a second.
  pub fn get_rate(&self) -> u32 {
    self.shared.rate.load(Ordering::Relaxed)
  }

  pub(crate) fn set_rate(&self, rate: u32) {
    self.shared.rate.store(rate, Ordering::Relaxed);
  }

  /// Where the mixer's frames play, none until a device or the pace of none has rendered it; an offline render sets
  /// none.
  pub fn get_anchor(&self) -> Option<AudioClockAnchor> {
    self.shared.anchor.read()
  }

  /// Anchors the clock at a device's buffer: its first frame, handed over now, plays `latency` later.
  pub(crate) fn anchor_at(&self, latency: Duration) {
    self.shared.anchor.write(AudioClockAnchor {
      frame: self.get_frames(),
      at: Instant::now(),
      latency,
    });
  }

  /// Anchors the clock at the next frame, rendered with no device for the time `at`, only where the anchor it has puts
  /// that frame more than `most_drift` away, so the pace of a render with no device does not shake it.
  pub(crate) fn follow_anchor(&self, at: Instant, most_drift: Duration) {
    let frame: u64 = self.get_frames();
    let rate: u32 = self.get_rate();
    let is_held: bool = self.get_anchor().is_some_and(|anchor| {
      let placed: Instant = anchor.get_instant(frame, rate);

      placed.max(at).duration_since(placed.min(at)) <= most_drift
    });

    if !is_held {
      self.shared.anchor.write(AudioClockAnchor {
        frame,
        at,
        latency: Duration::ZERO,
      });
    }
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
