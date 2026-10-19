use serde::{Deserialize, Serialize};

use crate::contract::render_reflection_mode::RenderReflectionMode;
use crate::contract::render_reflection_probe::RenderReflectionProbe;
use crate::contract::render_trace_size::RenderTraceSize;

/// What the frame reflects, in three parts: the surfaces' screen-space reflections (the ground, puddles, wet and glossy
/// surfaces), the enhanced water's own reflection of the screen (rivers and lakes), and the ray trace both take what the
/// screen misses from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderReflectionSettings {
  /// The surfaces' reflections: the engine's sky cube alone, or traced over the screen.
  pub mode: RenderReflectionMode,
  /// What scales the specular's share of what a surface reflects, one as its material reflects.
  pub intensity: f32,
  /// Metres a reflection ray goes at most, the surfaces' and the water's traces alike.
  pub distance: f32,
  /// Metres out to which the rain wets the ground's gloss for the rays to trace, fading over the last fifth; the
  /// engine alone wets it over 25 m.
  pub reach: f32,
  /// The size the surfaces' rays are traced at.
  pub trace_size: RenderTraceSize,
  /// Cells of the depth pyramid a surface's ray crosses at most, half of them for a rough one.
  pub steps: u32,
  /// How fine the surroundings cube a ray meeting nothing on the screen reads is.
  pub probe: RenderReflectionProbe,
  /// How far the sky's highlights are scaled at its white as light for the traced reflections, one as the engine's sky
  /// picture shows them.
  pub sky_gain: f32,
  /// Whether the rays meet the grass: walked over the depth as the grass left it, rather than as it stood before the
  /// grass drew, so blades and tufts stand in the reflections as solid as they draw, and stop rays behind them.
  pub is_grass_reflected: bool,
  /// The roughness past which a surface is not traced and reflects the prefiltered sky alone, the traced reflection
  /// fading into it over the last third: 0.6 by default, 0.4 as Unreal's Lumen traces.
  pub trace_roughness: f32,
  /// Whether a dry level's surfaces are traced where they show little of what they reflect, a dielectric's 4%; off,
  /// they reflect the prefiltered sky alone until the rain wets them, and only metal and glass-like surfaces trace.
  pub is_dry_traced: bool,
  /// Whether each quad of traced pixels takes rays by what it shows, as Battlefield V's and AMD's SSSR's budgets do:
  /// every pixel where a bright, sharp reflection or a puddle stands, two where it is dim or blurred, one where both or
  /// rough, the rest filled from their neighbours; off, every traced pixel walks its own ray.
  pub is_ray_budgeted: bool,
  /// Whether the enhanced water reflects the screen; off, it reflects the sky cube alone and keeps the rest of its look.
  pub is_water_reflected: bool,
  /// Steps the water's reflection marches over the screen.
  pub water_steps: u32,
  /// Whether a ray the screen does not answer is traced against the level's static geometry by hardware ray query,
  /// its hit lit, in place of the sky: the surfaces' while they are drawn, the water's while it reflects; on a device
  /// without ray queries it falls back to the sky as before.
  pub is_traced: bool,
  /// Frames a pixel the screen does not answer waits between its traced rays, the last answer held between.
  pub trace_interval: u32,
}

impl Default for RenderReflectionSettings {
  /// The engine's: the surfaces reflect the cube alone; the water reflects the screen; nothing is ray traced.
  fn default() -> Self {
    Self {
      mode: RenderReflectionMode::Engine,
      intensity: 1.0,
      distance: 150.0,
      reach: 300.0,
      trace_size: RenderTraceSize::Half,
      steps: Self::STEPS,
      probe: RenderReflectionProbe::Fine,
      sky_gain: 1.25,
      is_grass_reflected: false,
      trace_roughness: Self::TRACE_ROUGHNESS,
      is_dry_traced: false,
      is_ray_budgeted: true,
      is_water_reflected: true,
      water_steps: Self::WATER_STEPS,
      is_traced: false,
      trace_interval: Self::TRACE_INTERVAL,
    }
  }
}

impl RenderReflectionSettings {
  /// The surfaces' walk, by default, and the most and least it may be.
  pub const STEPS: u32 = 96;
  pub const STEPS_RANGE: (u32, u32) = (16, 128);
  /// The water's march, by default, and the most and least it may be.
  pub const WATER_STEPS: u32 = 16;
  pub const WATER_STEPS_RANGE: (u32, u32) = (8, 32);
  /// Frames between a missed pixel's traced rays, by default, and the most it may be.
  pub const TRACE_INTERVAL: u32 = 2;
  pub const TRACE_INTERVAL_MOST: u32 = 8;
  /// The roughness past which a surface is not traced, by default.
  pub const TRACE_ROUGHNESS: f32 = 0.6;

  /// Whether the surfaces' reflections are traced at all: enhanced, and their intensity and distance above none.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderReflectionMode::Enhanced && self.intensity > 0.0 && self.distance > 0.0
  }

  /// The surfaces' walk, held to its range.
  pub fn get_steps(&self) -> u32 {
    self.steps.clamp(Self::STEPS_RANGE.0, Self::STEPS_RANGE.1)
  }

  /// The water's march, held to its range.
  pub fn get_water_steps(&self) -> u32 {
    self
      .water_steps
      .clamp(Self::WATER_STEPS_RANGE.0, Self::WATER_STEPS_RANGE.1)
  }

  /// Frames between a missed pixel's traced rays, held to one to `TRACE_INTERVAL_MOST`.
  pub fn get_trace_interval(&self) -> u32 {
    self.trace_interval.clamp(1, Self::TRACE_INTERVAL_MOST)
  }
}
