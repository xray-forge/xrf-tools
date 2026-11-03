use serde::{Deserialize, Serialize};

/// An output device the system offers.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDeviceDescription {
  /// The device's id, stable across runs, which an output choice of one device names it by.
  pub id: String,
  pub name: String,
  /// Whether the system plays on it by default now.
  pub is_default: bool,
}
