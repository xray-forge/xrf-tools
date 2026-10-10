use serde::{Deserialize, Serialize};

/// How the level's surfaces are displaced by their height.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderParallaxSettings {
  /// `r2_steep_parallax`: the engines' steep parallax on a level surface whose texture asks for it.
  pub is_steep: bool,
}

impl Default for RenderParallaxSettings {
  /// Steep, as Anomaly's high and extreme presets set it.
  fn default() -> Self {
    Self { is_steep: true }
  }
}
