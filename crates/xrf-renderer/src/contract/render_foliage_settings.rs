use serde::{Deserialize, Serialize};

use crate::contract::render_foliage_mode::RenderFoliageMode;

/// How trees and grass move in the wind, and the enhanced motion's strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderFoliageSettings {
  pub mode: RenderFoliageMode,
  /// The least share of the strongest wind the foliage moves at, from nothing to one.
  pub min_speed: f32,
  /// How fast the grass's flow field drifts, how far it tosses the tufts, how far the wind pushes them downwind, and
  /// how much its gusts lift them.
  pub grass_speed: f32,
  pub grass_turbulence: f32,
  pub grass_push: f32,
  pub grass_wave: f32,
  /// How fast the branches' flow field drifts, how fast the trunks swing, and how far.
  pub trees_speed: f32,
  pub trees_trunk: f32,
  pub trees_bend: f32,
  /// How much sunlight leaves and grass pass through from behind, and how much of the sun's colour that light keeps.
  pub sss_intensity: f32,
  pub sss_color: f32,
}

impl Default for RenderFoliageSettings {
  /// The engine's own motion, with the enhanced motion's designed strengths.
  fn default() -> Self {
    Self {
      mode: RenderFoliageMode::Engine,
      min_speed: 0.1,
      grass_speed: 9.5,
      grass_turbulence: 1.4,
      grass_push: 1.5,
      grass_wave: 0.4,
      trees_speed: 11.0,
      trees_trunk: 0.15,
      trees_bend: 0.5,
      sss_intensity: 2.0,
      sss_color: 1.0,
    }
  }
}

impl RenderFoliageSettings {
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderFoliageMode::Enhanced
  }
}
