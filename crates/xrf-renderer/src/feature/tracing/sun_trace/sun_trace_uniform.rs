use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::pass::noise_turn::NoiseTurn;

/// The sun's angular radius, which a traced shadow's rays are drawn within: the real sun's, about half a degree across,
/// so a shadow's penumbra widens with its caster's distance as daylight's does.
const SUN_ANGULAR_RADIUS: f32 = 0.004_65;

/// What the sun's traced shadow reads, as `shaders/frame/sun_trace.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "SunTrace")]
pub struct SunTraceUniform {
  /// Towards the sun, in view space.
  pub to_sun: Vec4,
  /// The tangent of the sun's angular radius, which a ray's direction is drawn within.
  pub spread: f32,
  /// What this frame turns each pixel's noise by, each way.
  pub noise_x: f32,
  pub noise_y: f32,
  pub _pad: f32,
}

impl SunTraceUniform {
  /// Towards the sun's view space direction `to_sun`, on the noise's `frame`.
  pub fn new(to_sun: Vec4, frame: u32) -> Self {
    let [noise_x, noise_y] = NoiseTurn::get_r2(frame, NoiseTurn::FRAMES);

    Self {
      to_sun,
      spread: SUN_ANGULAR_RADIUS.tan(),
      noise_x,
      noise_y,
      _pad: 0.0,
    }
  }
}
