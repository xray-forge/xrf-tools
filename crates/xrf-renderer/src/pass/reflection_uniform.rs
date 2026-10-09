use xrf_renderer_core::ShaderStruct;

use crate::contract::render_reflection_settings::RenderReflectionSettings;

/// What the reflections' trace and filters read, as `shaders/frame/reflections.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Reflections")]
pub struct ReflectionUniform {
  /// Cells of the depth pyramid a ray crosses at most.
  pub crossings: u32,
  /// The frame's count, which turns each pixel's noise from one frame to the next.
  pub frame: u32,
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
  /// The settings, at a frame's count, with or without the last frame's reflections to carry.
  pub fn new(settings: &RenderReflectionSettings, frame: u32, has_history: bool) -> Self {
    let quality = settings.quality;

    Self {
      crossings: quality.get_crossings(),
      frame,
      intensity: settings.intensity.max(0.0),
      distance: settings.distance.max(0.0),
      has_history: f32::from(u8::from(has_history)),
      ratio: quality.get_ratio() as f32,
    }
  }
}
