use crate::contract::render_reflection_quality::RenderReflectionQuality;
use crate::contract::render_view_options::RenderViewOptions;

/// Whether a frame traces its screen-space reflections, and how: none skips every pass and drops the histories.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionTrace {
  pub quality: RenderReflectionQuality,
  /// What scales how much of a surface's reflection a traced one replaces, and how much a puddle's coat reflects.
  pub intensity: f32,
}

impl ReflectionTrace {
  pub const PYRAMID_PASS: &'static str = "reflection pyramid";
  pub const TRACE_PASS: &'static str = "reflection trace";
  pub const REPROJECT_PASS: &'static str = "reflection reproject";
  pub const AVERAGE_PASS: &'static str = "reflection average";
  pub const PREFILTER_PASS: &'static str = "reflection prefilter";
  pub const RESOLVE_PASS: &'static str = "reflection resolve";

  /// What the view traces, or none: unlit, wireframe, or the engine's own reflections asked for.
  pub fn new(options: &RenderViewOptions) -> Option<Self> {
    let settings = &options.features.reflections;

    if !options.mode.is_lit || options.mode.is_wireframe || !settings.is_drawn() {
      return None;
    }

    Some(Self {
      quality: settings.quality,
      intensity: settings.intensity,
    })
  }

  /// The frame's pixels a traced pixel stands for each way.
  pub fn get_ratio(self) -> u32 {
    self.quality.get_ratio()
  }
}
