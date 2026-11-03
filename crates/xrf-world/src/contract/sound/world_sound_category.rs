use serde::{Deserialize, Serialize};

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
