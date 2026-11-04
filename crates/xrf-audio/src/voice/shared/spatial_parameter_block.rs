use std::sync::atomic::{AtomicU32, Ordering};

use crate::voice::shared::voice_spatial_parameters::VoiceSpatialParameters;

/// A slot's [`VoiceSpatialParameters`] as atomics, each field written and read on its own: the control side rewrites
/// them all every tick, so a block reading half of one tick and half of the next hears no more than a tick's change.
#[derive(Debug)]
pub(crate) struct SpatialParameterBlock {
  values: [AtomicU32; Self::COUNT],
}

impl SpatialParameterBlock {
  const COUNT: usize = 13;

  pub(crate) fn get(&self) -> VoiceSpatialParameters {
    let value = |index: usize| f32::from_bits(self.values[index].load(Ordering::Relaxed));

    VoiceSpatialParameters {
      direction: [value(0), value(1), value(2)],
      attenuation: value(3),
      air_absorption: [value(4), value(5), value(6)],
      occlusion: value(7),
      transmission: [value(8), value(9), value(10)],
      reverb_send: value(11),
      is_binaural: value(12) != 0.0,
    }
  }

  /// Writes `parameters`, a value not a finite number taken as its default.
  pub(crate) fn set(&self, parameters: &VoiceSpatialParameters) {
    let defaults: VoiceSpatialParameters = VoiceSpatialParameters::default();
    let direction: [f32; 3] = if parameters.direction.iter().all(|it| it.is_finite()) {
      parameters.direction
    } else {
      defaults.direction
    };
    let values: [f32; Self::COUNT] = [
      direction[0],
      direction[1],
      direction[2],
      Self::to_share(parameters.attenuation),
      Self::to_share(parameters.air_absorption[0]),
      Self::to_share(parameters.air_absorption[1]),
      Self::to_share(parameters.air_absorption[2]),
      Self::to_share(parameters.occlusion),
      Self::to_share(parameters.transmission[0]),
      Self::to_share(parameters.transmission[1]),
      Self::to_share(parameters.transmission[2]),
      if parameters.reverb_send.is_finite() {
        parameters.reverb_send.max(0.0)
      } else {
        0.0
      },
      f32::from(u8::from(parameters.is_binaural)),
    ];

    for (slot, value) in self.values.iter().zip(values) {
      slot.store(value.to_bits(), Ordering::Relaxed);
    }
  }

  fn to_share(value: f32) -> f32 {
    if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 }
  }
}

impl Default for SpatialParameterBlock {
  fn default() -> Self {
    let block: Self = Self {
      values: std::array::from_fn(|_| AtomicU32::new(0)),
    };

    block.set(&VoiceSpatialParameters::default());
    block
  }
}
