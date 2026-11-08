use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Where a level's music stands: the track playing, how far into it and how long it is, or how long until the next is
/// picked, and whether a track was chosen by hand.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSoundMusicReport {
  /// The track playing, by its entry's name, none between tracks.
  pub track: Option<String>,
  /// How far it has played.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub position: Duration,
  /// How long it plays, nought between tracks.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub length: Duration,
  /// Level time until the next pick, zero while a track plays.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub wait: Duration,
  /// The track played by hand in place of the schedule, none while the schedule picks.
  pub chosen: Option<String>,
  /// Every track the level has, by name, sorted.
  pub tracks: Vec<String>,
}
