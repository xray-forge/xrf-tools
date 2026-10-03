use serde::{Deserialize, Serialize};

/// The sun's shadow as drawn.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAppliedShadows {
  /// Each cascade's width in metres, nearest first, as many as are drawn.
  pub cascades: Vec<f32>,
  /// Texels each cascade's map is across, held to what the device allows.
  pub resolution: u32,
  /// Texels the filter reaches each way.
  pub filter: u32,
}
