use serde::{Deserialize, Serialize};

use crate::contract::render_foliage_settings::RenderFoliageSettings;

/// The grass (`CDetailManager`): planted on the GPU around the camera as the engine plants it, and drawn into the
/// G-buffer. The engine's are 49 metres round at a density of 0.6 (`r__detail_radius`, `r__detail_density`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGrassSettings {
  pub is_enabled: bool,
  /// How far apart a slot's candidates stand, from 0.1 (the densest) to 0.99 (the sparsest): `r__detail_density`.
  pub density: f32,
  /// Whole metres around the camera grass is planted to: `r__detail_radius`.
  pub radius: f32,
  /// What every planted tuft is scaled by: `r__detail_height`.
  pub height: f32,
  /// How the trees and the grass move in the wind.
  pub foliage: RenderFoliageSettings,
}

impl Default for RenderGrassSettings {
  /// The engine's own grass.
  fn default() -> Self {
    Self {
      is_enabled: true,
      density: 0.6,
      radius: 49.0,
      height: 1.0,
      foliage: RenderFoliageSettings::default(),
    }
  }
}
