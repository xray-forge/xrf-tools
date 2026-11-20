use serde::{Deserialize, Serialize};

/// Whether a transport plays, as a media element's states name it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioTransportPlayback {
  /// At the start, or put back there.
  Stopped,
  Playing,
  /// Held where it was paused or sought to.
  Paused,
  /// Played to its end without looping; playing again starts over.
  Ended,
}
