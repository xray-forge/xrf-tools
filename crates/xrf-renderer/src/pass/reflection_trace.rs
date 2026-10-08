use xrf_engine_target::XrayEngine;

use crate::contract::render_reflection_quality::RenderReflectionQuality;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;

/// Whether a frame traces its screen-space reflections, and how: none skips every pass and drops the histories.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionTrace {
  pub quality: RenderReflectionQuality,
  /// How much of the cube a hit replaces.
  pub intensity: f32,
}

impl ReflectionTrace {
  pub const DEPTH_PASS: &'static str = "reflection depth";
  pub const TRACE_PASS: &'static str = "reflection trace";
  pub const ACCUMULATE_PASS: &'static str = "reflection accumulate";
  pub const FILTER_PASS: &'static str = "reflection filter";

  /// What the view traces, or none: unlit, wireframe, the engine's own reflections asked for, or Anomaly's shading
  /// while it is dry, which reflects nothing for a ray to replace.
  pub fn new(options: &RenderViewOptions, lighting: &RenderLighting) -> Option<Self> {
    let settings = &options.features.reflections;

    if !options.mode.is_lit || options.mode.is_wireframe || !settings.is_drawn() {
      return None;
    }

    if lighting.engine == XrayEngine::Extended && lighting.rain.is_none_or(|rain| rain.density <= 0.0) {
      return None;
    }

    Some(Self {
      quality: settings.quality,
      intensity: settings.intensity.min(1.0),
    })
  }

  /// The frame's pixels a traced pixel stands for each way.
  pub fn get_ratio(self) -> u32 {
    self.quality.get_ratio()
  }
}
