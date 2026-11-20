use serde::{Deserialize, Serialize};
use xrf_audio::AudioBus;

/// How loud each category plays under the master, each from 0 (silent) to 1 (as recorded), linear as a media element's
/// volume is.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioVolumes {
  pub master: f32,
  /// A level's music tracks.
  pub music: f32,
  /// Static sounds, the weather's ambient channels, rain, thunder and ambient effects.
  pub ambience: f32,
  /// Sounds at the world's objects: anomalies and campfires.
  pub world: f32,
  /// Sounds a tool previews, such as an archived sound.
  pub previews: f32,
}

impl AudioVolumes {
  pub fn get_bus(&self, bus: AudioBus) -> f32 {
    match bus {
      AudioBus::Music => self.music,
      AudioBus::Ambience => self.ambience,
      AudioBus::World => self.world,
      AudioBus::Previews => self.previews,
    }
  }

  /// The volumes held to `[0, 1]`, one that is not a number at full volume.
  pub fn clamp(&self) -> Self {
    let clamp = |volume: f32| {
      if volume.is_finite() {
        volume.clamp(0.0, 1.0)
      } else {
        1.0
      }
    };

    Self {
      master: clamp(self.master),
      music: clamp(self.music),
      ambience: clamp(self.ambience),
      world: clamp(self.world),
      previews: clamp(self.previews),
    }
  }
}

impl Default for AudioVolumes {
  fn default() -> Self {
    Self {
      master: 1.0,
      music: 1.0,
      ambience: 1.0,
      world: 1.0,
      previews: 1.0,
    }
  }
}
