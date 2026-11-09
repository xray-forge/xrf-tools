use std::sync::atomic::{AtomicU32, Ordering};

use crate::voice::shared::spatial_parameter_block::SpatialParameterBlock;
use crate::voice::shared::voice_spatial_parameters::VoiceSpatialParameters;

/// A voice's continuous controls, in a block per slot that the control side writes and the audio thread reads once a
/// block. Unlike a command, a write never waits on the audio thread and is never refused, and writes between two
/// blocks coalesce into the last.
#[derive(Debug)]
pub(crate) struct VoiceParameters {
  /// Linear gain, as `f32` bits, zero or above.
  gain: AtomicU32,
  /// Playback speed, as `f32` bits, above zero.
  rate: AtomicU32,
  /// Linear gains of the left and right outputs, as `f32` bits, zero or above: where a dry placed voice is heard.
  pan: [AtomicU32; 2],
  /// How a voice the acoustics place is heard.
  spatial: SpatialParameterBlock,
}

impl VoiceParameters {
  pub(crate) fn get_gain(&self) -> f32 {
    f32::from_bits(self.gain.load(Ordering::Relaxed))
  }

  /// Sets the gain; one below zero or not a number is silence.
  pub(crate) fn set_gain(&self, gain: f32) {
    self.gain.store(Self::sanitize_gain(gain).to_bits(), Ordering::Relaxed);
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

  pub(crate) fn get_pan(&self) -> [f32; 2] {
    self
      .pan
      .each_ref()
      .map(|gain| f32::from_bits(gain.load(Ordering::Relaxed)))
  }

  /// Sets the left and right gains; one below zero or not a number is silence on that side.
  pub(crate) fn set_pan(&self, pan: [f32; 2]) {
    for (gain, value) in self.pan.iter().zip(pan) {
      gain.store(Self::sanitize_gain(value).to_bits(), Ordering::Relaxed);
    }
  }

  pub(crate) fn get_spatial(&self) -> VoiceSpatialParameters {
    self.spatial.get()
  }

  pub(crate) fn set_spatial(&self, parameters: &VoiceSpatialParameters) {
    self.spatial.set(parameters);
  }

  fn sanitize_gain(gain: f32) -> f32 {
    if gain.is_finite() { gain.max(0.0) } else { 0.0 }
  }
}

impl Default for VoiceParameters {
  fn default() -> Self {
    Self {
      gain: AtomicU32::new(1f32.to_bits()),
      rate: AtomicU32::new(1f32.to_bits()),
      pan: [AtomicU32::new(1f32.to_bits()), AtomicU32::new(1f32.to_bits())],
      spatial: SpatialParameterBlock::default(),
    }
  }
}
