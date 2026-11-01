use glam::{Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_contact_shadow_settings::RenderContactShadowSettings;
use crate::pass::noise_turn::NoiseTurn;

/// The share of the drawn height a ray spans at most, however near the surface it leaves.
const MAX_REACH: f32 = 0.125;

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
  /// Frame texels across each texel of the sun's target stands for: one at the frame's size, two at half of it.
  pub stride: u32,
  /// Metres from the eye grass hides a ray to, none where the grass casts none, and from where it fades out; unused.
  pub grass: Vec4,
}

impl ContactShadowUniform {
  /// The settings towards the sun and as many local lights as they march towards, over a drawing `height` pixels tall,
  /// on the noise's `frame`, grass hiding a ray within `grass` metres (`ContactShadowUniform::grass`).
  pub fn new(settings: &RenderContactShadowSettings, to_sun: Vec4, height: u32, frame: u32, grass: Vec2) -> Self {
    Self {
      to_sun,
      length: settings.length,
      intensity: settings.intensity.min(1.0),
      thickness: settings.thickness,
      steps: settings.steps,
      reach: (height as f32 * MAX_REACH).max(1.0),
      noise: NoiseTurn::get_gradient(frame, NoiseTurn::FRAMES),
      lights: settings.get_light_count(),
      stride: if settings.is_full_resolution { 1 } else { 2 },
      grass: grass.extend(0.0).extend(0.0),
    }
  }
}
