use serde::{Deserialize, Serialize};

/// How a terrain's four detail textures repeat across the ground.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderTerrainTiling {
  /// The engines' own: each detail repeating as a grid, every few metres the same.
  #[default]
  Engine,
  /// Hex tiling (Mikkelsen, "Practical Real-Time Hex-Tiling", JCGT 2022): each detail read in three randomly offset and
  /// turned copies blended over a grid of hexagons, so no repeat lines up with the next.
  Varied,
}
