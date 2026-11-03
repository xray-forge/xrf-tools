use serde::{Deserialize, Serialize};

/// Which device the audio plays on.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "kind")]
pub enum AudioOutputChoice {
  /// The system's default device, followed as the system changes it.
  #[default]
  SystemDefault,
  /// One device by its id; the system default plays while it is missing.
  Device { id: String },
}
