use serde::{Deserialize, Serialize};

/// What the output plays on now.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioOutputStatus {
  /// The device playing, or `None` while none could be opened.
  pub device_name: Option<String>,
  pub sample_rate: u32,
  pub channels: u16,
  /// Whether the system default plays because the chosen device is missing.
  pub is_fallback: bool,
  /// Why no device plays, while none does.
  pub failure: Option<String>,
}
