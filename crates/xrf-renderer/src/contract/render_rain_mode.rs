use serde::{Deserialize, Serialize};

/// How rain wets the frame's surfaces.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderRainMode {
  /// The engine's own: splashes and running water near the camera, wet only while it rains.
  #[default]
  Engine,
  /// Wet surfaces that build up and dry over time, rippling and running with water out to the distance, and puddles
  /// gathering on flat, low terrain.
  Enhanced,
}
