use xrf_renderer_core::ShaderStruct;

use crate::contract::render_reflection_settings::RenderReflectionSettings;

/// What the reflections' trace, blend over frames and blurs read, as `shaders/frame/reflections.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Reflections")]
pub struct ReflectionUniform {
  /// Steps a ray takes at most.
  pub steps: u32,
  /// Metres behind a surface a step may land and still have met it.
  pub limit: f32,
  /// One where a step landing further is halved back once.
  pub is_refined: f32,
  /// What a surface's gloss and Fresnel term are scaled by into its share of reflection.
  pub intensity: f32,
  /// Metres a ray is traced at most.
  pub distance: f32,
  /// One where the last frame's reflections are there to carry.
  pub has_history: f32,
  /// The frame's pixels a traced pixel stands for each way.
  pub ratio: f32,
}

impl ReflectionUniform {
  /// The settings, with or without the last frame's reflections to carry.
  pub fn new(settings: &RenderReflectionSettings, has_history: bool) -> Self {
    let quality = settings.quality;

    Self {
      steps: quality.get_steps(),
      limit: quality.get_limit(),
      is_refined: f32::from(u8::from(quality.is_refined())),
      intensity: settings.intensity.max(0.0),
      distance: settings.distance.max(0.0),
      has_history: f32::from(u8::from(has_history)),
      ratio: quality.get_ratio() as f32,
    }
  }
}
