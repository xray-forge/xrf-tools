use std::time::Duration;

use serde::{Deserialize, Serialize};

/// One sound channel of the ambient playing and how long until its next sound.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSoundChannelReport {
  /// The `sound_channels.ltx` section.
  pub channel: String,
  /// Level time until its next sound.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub wait: Duration,
}
