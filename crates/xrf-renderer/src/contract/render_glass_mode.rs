use serde::{Deserialize, Serialize};

/// How glass is drawn.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderGlassMode {
  /// The engine's own: its base blended over what is behind it, mixed toward the sky's cube where it is thin.
  #[default]
  Engine,
  /// Physically based glass: what stands behind it refracted, blurred where it is dirty, the sky reflected by the
  /// fresnel and the sun's highlight on it.
  Enhanced,
}
