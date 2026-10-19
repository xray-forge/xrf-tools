use serde::{Deserialize, Serialize};

use crate::contract::render_reflection_mode::RenderReflectionMode;
use crate::contract::render_reflection_probe::RenderReflectionProbe;
use crate::contract::render_reflection_quality::RenderReflectionQuality;
use crate::contract::render_reflection_trace_quality::RenderReflectionTraceQuality;

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
  /// How far the sky's highlights are scaled at its white as light for the traced reflections, one as the engine's sky
  /// picture shows them.
  pub sky_gain: f32,
  /// Whether the rays meet the grass: walked over the depth as the grass left it, rather than as it stood before the
  /// grass drew, so blades and tufts stand in the reflections as solid as they draw, and stop rays behind them.
  pub is_grass_reflected: bool,
  /// Whether a ray the screen does not answer is traced against the level's static geometry by hardware ray query,
  /// its hit lit, in place of the sky; on a device without ray queries it falls back to the sky as before.
  pub is_traced: bool,
  pub trace_quality: RenderReflectionTraceQuality,
  /// The roughness past which a surface is not traced and reflects the prefiltered sky alone, the traced reflection
  /// fading into it over the last third: 0.6 by default, 0.4 as Unreal's Lumen traces.
  pub trace_roughness: f32,
  /// Whether each quad of traced pixels takes rays by what it shows, as Battlefield V's and AMD's SSSR's budgets do:
  /// every pixel where a bright, sharp reflection or a puddle stands, two where it is dim or blurred, one where both or
  /// rough, the rest filled from their neighbours; off, every traced pixel walks its own ray.
  pub is_ray_budgeted: bool,
}

impl Default for RenderReflectionSettings {
  /// Off: the engine reflects the cube alone.
  fn default() -> Self {
    Self {
      mode: RenderReflectionMode::Engine,
      intensity: 1.0,
      distance: 150.0,
      reach: 300.0,
      quality: RenderReflectionQuality::High,
      probe: RenderReflectionProbe::Fine,
      sky_gain: 1.25,
      is_grass_reflected: false,
      is_traced: false,
      trace_quality: RenderReflectionTraceQuality::High,
      trace_roughness: Self::TRACE_ROUGHNESS,
      is_ray_budgeted: false,
    }
  }
}

impl RenderReflectionSettings {
  /// The roughness past which a surface is not traced, by default.
  pub const TRACE_ROUGHNESS: f32 = 0.6;

  /// Whether it is traced at all: enhanced, and its intensity and distance above none.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderReflectionMode::Enhanced && self.intensity > 0.0 && self.distance > 0.0
  }
}
