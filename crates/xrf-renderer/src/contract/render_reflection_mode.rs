use serde::{Deserialize, Serialize};

/// What a glossy surface reflects: the sky's cube alone, or what the frame shows where a ray finds it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderReflectionMode {
  /// The engine's own: the irradiance cube along the reflection, weighed by the surface's gloss.
  #[default]
  Engine,
  /// Screen-space reflections: each glossy pixel's reflected ray traced over the frame's depth, what it meets in place
  /// of the cube, the cube where it meets nothing.
  Enhanced,
}
