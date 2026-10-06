use serde::{Deserialize, Serialize};

use crate::contract::render_reflection_mode::RenderReflectionMode;
use crate::contract::render_reflection_probe::RenderReflectionProbe;
use crate::contract::render_reflection_quality::RenderReflectionQuality;

/// Screen-space reflections on the frame's surfaces: what each surface's reflected rays meet over the frame, or the
/// sky prefiltered to its roughness where they meet nothing, shown by a physically based specular in place of the
/// engine's reflection of its cube.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderReflectionSettings {
  pub mode: RenderReflectionMode,
  /// What scales the specular's share of what a surface reflects, one as its material reflects.
  pub intensity: f32,
  /// Metres a ray is traced at most.
  pub distance: f32,
  /// Metres out to which the rain wets the ground's gloss for the rays to trace, fading over the last fifth; the
  /// engine alone wets it over 25 m.
  pub reach: f32,
  pub quality: RenderReflectionQuality,
  /// How fine the surroundings cube a ray meeting nothing on the screen reads is.
  pub probe: RenderReflectionProbe,
}

impl Default for RenderReflectionSettings {
  /// Off: the engine reflects the cube alone.
  fn default() -> Self {
    Self {
      mode: RenderReflectionMode::Engine,
      intensity: 1.0,
      distance: 150.0,
      reach: 250.0,
      quality: RenderReflectionQuality::High,
      probe: RenderReflectionProbe::Fine,
    }
  }
}

impl RenderReflectionSettings {
  /// Whether it is traced at all: enhanced, and its intensity and distance above none.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderReflectionMode::Enhanced && self.intensity > 0.0 && self.distance > 0.0
  }
}
