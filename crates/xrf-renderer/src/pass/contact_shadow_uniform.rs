use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_contact_shadow_settings::RenderContactShadowSettings;

/// The share of the drawn height a ray spans at most, however near the surface it leaves.
const MAX_REACH: f32 = 0.125;

/// Interleaved gradient noise's step between frames, Jimenez's, in pixels.
const NOISE_STEP: f32 = 5.588_238;

/// Frames the noise cycles through before it repeats.
const NOISE_FRAMES: u32 = 64;

/// What the contact shadows' march reads, as `shaders/common/contact_march.wgsl` takes it: the sun's pass and the
/// local lights' alike.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "ContactShadows")]
pub struct ContactShadowUniform {
  /// Towards the sun, in view space.
  pub to_sun: Vec4,
  pub length: f32,
  pub intensity: f32,
  pub thickness: f32,
  pub steps: u32,
  /// Drawn pixels a ray spans at most.
  pub reach: f32,
  /// Pixels the noise is moved by this frame.
  pub noise: f32,
  /// Local lights each pixel marches towards at most: none for the sun's alone.
  pub lights: u32,
  pub _pad: f32,
}

impl ContactShadowUniform {
  /// The settings towards the sun and as many local lights as they march towards, over a drawing `height` pixels tall,
  /// on the noise's `frame`.
  pub fn new(settings: &RenderContactShadowSettings, to_sun: Vec4, height: u32, frame: u32) -> Self {
    Self {
      to_sun,
      length: settings.length,
      intensity: settings.intensity.min(1.0),
      thickness: settings.thickness,
      steps: settings.steps,
      reach: (height as f32 * MAX_REACH).max(1.0),
      noise: (frame % NOISE_FRAMES) as f32 * NOISE_STEP,
      lights: settings.get_light_count(),
      _pad: 0.0,
    }
  }
}
