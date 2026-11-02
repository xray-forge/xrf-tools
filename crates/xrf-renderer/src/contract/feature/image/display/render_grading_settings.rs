use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

/// How the physical light's frame is graded before its display transform, in scene-referred light: its contrast about
/// middle grey, a power of its luminance in stops, and its saturation, a mix from its own luminance; one leaves each.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGradingSettings {
  // From half the contrast to twice it, and from grey to twice the colour.
  #[setting(min = 0.5, max = 2)]
  pub contrast: f32,
  #[setting(min = 0, max = 2)]
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
