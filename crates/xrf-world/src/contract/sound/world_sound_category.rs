use serde::{Deserialize, Serialize};
use xrf_audio_scene::AudioBus;

/// The volumes a level's sounds are heard at, which the viewer switches each on or off.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WorldSoundCategory {
  Music,
  /// Static sounds, the weather's channels, rain, thunder and ambient effects.
  Ambience,
  /// Sounds at the world's objects: anomalies and campfires.
  World,
}

impl WorldSoundCategory {
  pub const ALL: [Self; 3] = [Self::Music, Self::Ambience, Self::World];

  /// The audio's bus a sound of the category plays on, its volume the category's.
  pub fn get_bus(self) -> AudioBus {
    match self {
      Self::Music => AudioBus::Music,
      Self::Ambience => AudioBus::Ambience,
      Self::World => AudioBus::World,
    }
  }
}
