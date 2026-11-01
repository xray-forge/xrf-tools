use serde::{Deserialize, Serialize};

/// How a shadowed local light's map is compared.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderLightShadowFilter {
  /// `shadow_hw`: four bilinear comparisons 0.6 of a texel off the point, at `r2_ls_depth_bias` -0.0003, as vanilla.
  #[default]
  Engine,
  /// Anomaly's `shadow_pcss`: a blocker search, then a penumbra of twelve comparisons, at its -0.001 bias.
  Soft,
}
