use serde::{Deserialize, Serialize};

/// The weather effect a viewport plays over its cycle.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldWeatherEffectReport {
  pub name: String,
  /// Game seconds until the cycle takes over again.
  pub remaining: f32,
}
