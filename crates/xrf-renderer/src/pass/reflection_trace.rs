use crate::contract::render_reflection_quality::RenderReflectionQuality;
use crate::contract::render_view_options::RenderViewOptions;

/// Whether a frame traces its screen-space reflections, and how: none skips every pass and drops the histories.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionTrace {
  pub quality: RenderReflectionQuality,
  /// What scales how much of a surface's reflection a traced one replaces, and how much a puddle's coat reflects.
  pub intensity: f32,
  /// How far the sky's highlights are scaled as their light, from `SKY_GAIN_LEAST` to `SKY_GAIN_MOST`.
  pub sky_gain: f32,
  /// Whether DLSS ray reconstruction takes the trace raw: every pixel traced every frame and none of the denoiser's
  /// stages run.
  pub is_raw: bool,
}

impl ReflectionTrace {
  pub const PYRAMID_PASS: &'static str = "reflection pyramid";
  pub const TRACE_PASS: &'static str = "reflection trace";
  pub const SCENE_PASS: &'static str = "reflection scene";
  pub const REPROJECT_PASS: &'static str = "reflection reproject";
  pub const AVERAGE_PASS: &'static str = "reflection average";
  pub const PREFILTER_PASS: &'static str = "reflection prefilter";
  pub const RESOLVE_PASS: &'static str = "reflection resolve";
  pub const UPSCALE_PASS: &'static str = "reflection upscale";
  pub const GUIDE_PASS: &'static str = "reflection guide";
  /// The least and the most the sky's highlights are scaled by as light.
  pub const SKY_GAIN_LEAST: f32 = 0.5;
  pub const SKY_GAIN_MOST: f32 = 3.0;

  /// What the view traces, or none: unlit, wireframe, or the engine's own reflections asked for.
  pub fn new(options: &RenderViewOptions) -> Option<Self> {
    let settings = &options.features.reflections;

    if !options.mode.is_lit || options.mode.is_wireframe || !settings.is_drawn() {
      return None;
    }

    Some(Self {
      quality: settings.quality,
      intensity: settings.intensity,
      sky_gain: settings.sky_gain.clamp(Self::SKY_GAIN_LEAST, Self::SKY_GAIN_MOST),
      is_raw: false,
    })
  }

  /// The same trace, taken raw where DLSS ray reconstruction resolves the frame (`is_reconstructed`).
  pub fn with_raw(self, is_reconstructed: bool) -> Self {
    Self {
      is_raw: is_reconstructed,
      ..self
    }
  }

  /// The frame's pixels a traced pixel stands for each way: one taken raw, whatever the quality.
  pub fn get_ratio(self) -> u32 {
    if self.is_raw { 1 } else { self.quality.get_ratio() }
  }
}
