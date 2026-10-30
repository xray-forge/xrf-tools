use serde::{Deserialize, Serialize};

use crate::contract::render_terrain_tiling::RenderTerrainTiling;

/// How a level's terrain lays its detail textures over its ground.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderTerrainSettings {
  /// Whether the details repeat as the engines lay them or are varied by hex tiling, whose hexagons the rest shape.
  pub tiling: RenderTerrainTiling,
  /// How small the varied tiling's hexagons are, its grid's rate: at two its corners stand about a seventh of a
  /// detail's repeat apart, small enough that the copies' shifts scatter a fine repeated feature rather than move whole
  /// rows of it.
  pub scale: f32,
  /// How far each copy turns, from not at all to up to a half turn either way. The details carry light and shade
  /// painted in, which a turned copy turns with it: even a fifth mottled Zaton's crater with hexagons lit from
  /// different sides (2026-10-30), so none by default.
  pub rotation: f32,
  /// How sharply the copies meet, the corners' nearness raised to it: lower blends them over wider seams, softer and
  /// blurrier; higher keeps each copy whole to a crisp seam. Seven is the hex tiling demo's.
  pub sharpness: f32,
}

impl Default for RenderTerrainSettings {
  fn default() -> Self {
    Self {
      tiling: RenderTerrainTiling::Engine,
      scale: 2.0,
      rotation: 0.0,
      sharpness: 7.0,
    }
  }
}

impl RenderTerrainSettings {
  /// Whether the details are read by hex tiles rather than repeating as a grid.
  pub fn is_varied(&self) -> bool {
    self.tiling == RenderTerrainTiling::Varied
  }
}
