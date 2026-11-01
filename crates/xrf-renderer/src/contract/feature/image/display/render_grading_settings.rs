use serde::{Deserialize, Serialize};

/// How the physical light's frame is graded before its display transform, in scene-referred light: its contrast about
/// middle grey, a power of its luminance in stops, and its saturation, a mix from its own luminance; one leaves each.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGradingSettings {
  pub contrast: f32,
  pub saturation: f32,
}

impl Default for RenderGradingSettings {
  /// Nothing graded.
  fn default() -> Self {
    Self {
      contrast: 1.0,
      saturation: 1.0,
    }
  }
}
