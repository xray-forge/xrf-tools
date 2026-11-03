use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::transport::audio_transport_playback::AudioTransportPlayback;

/// Where a transport is, for a player to show.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTransportState {
  pub playback: AudioTransportPlayback,
  /// How far into the sound it plays or holds.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub position: Duration,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  pub is_looped: bool,
  /// Why playing failed: the stream stopped decoding, or the audio thread refused the voice.
  pub failure: Option<String>,
}
