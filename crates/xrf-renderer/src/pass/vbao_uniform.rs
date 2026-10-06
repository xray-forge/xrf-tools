use glam::Mat4;
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;

/// The exponent a strength of one raises the filtered visibility to, chosen by looking: creases read without blackening.
const BASE_POWER: f32 = 1.5;

/// The share of the search target's height a slice is searched across at most, however near the point.
const MAX_REACH: f32 = 0.2;

/// The R2 sequence's steps between frames (Roberts, 2018): the plastic number's reciprocal and its square.
const NOISE_STEPS: (f32, f32) = (0.754_877_7, 0.569_840_3);

/// Frames the noise cycles through before it repeats.
const NOISE_FRAMES: u32 = 256;

/// What VBAO's search, accumulation and filter read, as
/// `shaders/frame/vbao.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Vbao")]
pub struct VbaoUniform {
  pub radius: f32,
  pub thickness: f32,
  /// The exponent the filtered visibility is raised to.
  pub power: f32,
  /// Metres one pixel of the search target spans at a metre from the camera.
  pub spread: f32,
  /// Pixels of the search target a slice is searched across at most.
  pub reach: f32,
  /// Frames each pixel is averaged over at most; one for none.
  pub frames: f32,
  /// One where the last frame's accumulation is there to carry.
  pub has_history: f32,
  /// What this frame turns the slices' noise by, from zero to one.
  pub slice_noise: f32,
  /// What this frame moves the steps' noise by, from zero to one.
  pub step_noise: f32,
}

impl VbaoUniform {
  /// The settings over a search target of a size, through the camera's projection, on the noise's `frame`, with or
  /// without a history to carry.
  pub fn new(
    settings: &RenderAmbientOcclusionSettings,
    projection: Mat4,
    (width, height): (u32, u32),
    frame: u32,
    has_history: bool,
  ) -> Self {
    let frames: u32 = settings.vbao.get_accumulation();
    // A noise held still where nothing gathers it, so it never crawls.
    let turn: f32 = if frames > 1 { (frame % NOISE_FRAMES) as f32 } else { 0.0 };

    Self {
      radius: settings.radius,
      thickness: settings.vbao.thickness.max(0.0),
      power: BASE_POWER * settings.strength.max(0.0),
      // Clip x over view x at a metre is the projection's first element.
      spread: 2.0 / (projection.x_axis.x * width.max(1) as f32),
      reach: (height as f32 * MAX_REACH).max(1.0),
      frames: frames as f32,
      has_history: f32::from(u8::from(has_history && frames > 1)),
      slice_noise: (turn * NOISE_STEPS.0).fract(),
      step_noise: (turn * NOISE_STEPS.1).fract(),
    }
  }
}
