use serde::{Deserialize, Serialize};

/// Whether this machine can record a viewport, and why not where it cannot: no Media Foundation, or no audio.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecordingAvailability {
  pub is_available: bool,
  pub reason: Option<String>,
}
