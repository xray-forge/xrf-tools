use serde::{Deserialize, Serialize};

/// Distance fog as drawn.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAppliedFog {
  pub color: [f32; 3],
  /// Metres to where it is total, or the far plane where that is nearer.
  pub distance: f32,
  /// How near the camera it starts, a share of its distance.
  pub density: f32,
}
