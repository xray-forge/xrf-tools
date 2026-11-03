use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Where a level's music stands: the track playing and how far into it, or how long until the next is picked.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSoundMusicReport {
  /// The track playing, by its entry's name, none between tracks.
  pub track: Option<String>,
  /// How far it has played.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub position: Duration,
  /// Level time until the next pick, zero while a track plays.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub wait: Duration,
  /// Tracks the level has.
  pub tracks: u32,
}
