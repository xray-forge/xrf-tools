use serde::{Deserialize, Serialize};

/// How a viewport's weather clock runs.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldWeatherControl {
  /// Game seconds a real second, the engine's time factor.
  pub factor: f32,
  pub is_paused: bool,
  /// Whether a vanilla cycle stands the sun astronomically rather than by its keyframes.
  pub is_dynamic_sun: bool,
}

impl Default for WorldWeatherControl {
  /// Paused, at the engine's own time factor, with the sun the keyframes stand.
  fn default() -> Self {
    Self {
      factor: 12.0,
      is_paused: true,
      is_dynamic_sun: false,
    }
  }
}
