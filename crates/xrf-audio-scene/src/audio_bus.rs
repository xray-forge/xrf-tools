use serde::{Deserialize, Serialize};

/// The categories a voice plays in, each with its own volume before the master's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioBus {
  /// A level's music tracks.
  Music,
  /// Static sounds, the weather's ambient channels, rain, thunder and ambient effects.
  Ambience,
  /// Sounds at the world's objects: anomalies and campfires.
  World,
  /// Sounds an editor plays to preview them.
  Previews,
}

impl AudioBus {
  pub const ALL: [Self; 4] = [Self::Music, Self::Ambience, Self::World, Self::Previews];

  pub const COUNT: usize = Self::ALL.len();

  /// The bus's place in [`Self::ALL`].
  pub fn get_index(self) -> usize {
    match self {
      Self::Music => 0,
      Self::Ambience => 1,
      Self::World => 2,
      Self::Previews => 3,
    }
  }
}
