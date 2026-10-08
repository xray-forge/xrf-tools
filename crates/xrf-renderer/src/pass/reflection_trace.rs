use crate::contract::render_reflection_quality::RenderReflectionQuality;
use crate::contract::render_view_options::RenderViewOptions;

/// Whether a frame traces its screen-space reflections, and how: none skips every pass and drops the histories.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionTrace {
  pub quality: RenderReflectionQuality,
  /// What a surface's gloss and Fresnel term are scaled by into its share of reflection.
  pub intensity: f32,
}

impl ReflectionTrace {
  pub const TRACE_PASS: &'static str = "reflection trace";
  pub const ACCUMULATE_PASS: &'static str = "reflection accumulate";
  pub const BLUR_PASS: &'static str = "reflection blur";
  pub const FINE_BLUR_PASS: &'static str = "reflection blur fine";

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
