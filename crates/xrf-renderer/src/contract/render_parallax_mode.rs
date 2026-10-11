use serde::{Deserialize, Serialize};

/// Whose parallax displaces the level's surfaces.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderParallaxMode {
  /// The engines' own: steep parallax on a level surface whose texture asks for it, none on terrain.
  #[default]
  Engine,
  /// Terrain marched over its details' heights, each layer by its own, self-shadowed towards the sun, its layers blended
  /// by height.
  Enhanced,
}
