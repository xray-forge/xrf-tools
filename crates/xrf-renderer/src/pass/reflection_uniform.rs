use glam::Vec2;
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_reflection_settings::RenderReflectionSettings;

/// Frames each traced pixel is averaged over at most.
const ACCUMULATION: f32 = 8.0;

/// Interleaved gradient noise's step between frames, Jimenez's, in pixels.
const NOISE_STEP: f32 = 5.588_238;

/// Frames the noise cycles through before it repeats.
const NOISE_FRAMES: u32 = 64;

/// What the reflections' trace, accumulation and filter read, as `shaders/frame/reflections.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Reflections")]
pub struct ReflectionUniform {
  /// The traced depth's texels across the viewport each way: its size over the ratio.
  pub base: Vec2,
  /// Steps a ray takes at most.
  pub steps: u32,
  /// How much of the cube a hit replaces.
  pub intensity: f32,
  /// Metres a ray is traced at most.
  pub distance: f32,
  /// Frames each traced pixel is averaged over at most.
  pub frames: f32,
  /// One where the last frame's accumulation is there to carry.
  pub has_history: f32,
  /// Pixels the noise is moved by this frame.
  pub noise: f32,
  /// The frame's pixels a traced pixel stands for each way.
  pub ratio: f32,
  pub _pad: f32,
}

impl ReflectionUniform {
  /// The settings over a viewport of a size, on the noise's `frame`, with or without the last frame's accumulation to
  /// carry.
  pub fn new(settings: &RenderReflectionSettings, (width, height): (u32, u32), frame: u32, has_history: bool) -> Self {
    let ratio: u32 = settings.quality.get_ratio();

    Self {
      base: Vec2::new(width as f32, height as f32) / ratio as f32,
      steps: settings.quality.get_steps(),
      intensity: settings.intensity.clamp(0.0, 1.0),
      distance: settings.distance.max(0.0),
      frames: ACCUMULATION,
      has_history: f32::from(u8::from(has_history)),
      noise: (frame % NOISE_FRAMES) as f32 * NOISE_STEP,
      ratio: ratio as f32,
      _pad: 0.0,
    }
  }
}
