use crate::mixer::audio_bus::AudioBus;

/// The linear gains of the master and of each bus, set together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioGains {
  pub master: f32,
  /// Each bus's gain, at its place in [`AudioBus::ALL`].
  pub buses: [f32; AudioBus::COUNT],
}

impl AudioGains {
  pub fn get_bus(&self, bus: AudioBus) -> f32 {
    self.buses[bus.get_index()]
  }
}

impl Default for AudioGains {
  fn default() -> Self {
    Self {
      master: 1.0,
      buses: [1.0; AudioBus::COUNT],
    }
  }
}
