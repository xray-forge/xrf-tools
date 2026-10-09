use serde::{Deserialize, Serialize};

use crate::contract::render_reflection_mode::RenderReflectionMode;
use crate::contract::render_reflection_quality::RenderReflectionQuality;

/// Screen-space reflections on the frame's surfaces: what each glossy surface's reflected rays meet, or the sky where
/// they meet nothing, sharpening the slice of its own reflection a dielectric would show, and reflected by a puddle's
/// clear coat of water.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderReflectionSettings {
  pub mode: RenderReflectionMode,
  /// What scales how much of a surface's reflection a traced one replaces, and how much a puddle's coat reflects.
  pub intensity: f32,
  /// Metres a ray is traced at most.
  pub distance: f32,
  pub quality: RenderReflectionQuality,
}

impl Default for RenderReflectionSettings {
  /// Off: the engine reflects the cube alone.
  fn default() -> Self {
    Self {
      mode: RenderReflectionMode::Engine,
      intensity: 1.0,
      distance: 150.0,
      quality: RenderReflectionQuality::High,
    }
  }
}

impl RenderReflectionSettings {
  /// Whether it is traced at all: enhanced, and its intensity and distance above none.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderReflectionMode::Enhanced && self.intensity > 0.0 && self.distance > 0.0
  }
}
