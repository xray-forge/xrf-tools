use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::pass::noise_turn::NoiseTurn;

/// Metres about a lamp's position its light is taken to leave from, which a traced shadow's rays are drawn over: a
/// bulb's, so a shadow's penumbra widens with its caster's distance from the lamp.
const LAMP_RADIUS: f32 = 0.05;

/// What the lamps' traced shadows read, as `shaders/frame/light_tracing.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "LightTrace")]
pub struct LightTraceUniform {
  /// What this frame turns each pixel's noise by, each way, then nothing.
  pub noise: Vec4,
  /// Metres a lamp's light leaves from about its position.
  pub radius: f32,
  pub _pad: [f32; 3],
}

impl LightTraceUniform {
  /// On the noise's `frame`.
  pub fn new(frame: u32) -> Self {
    let [x, y] = NoiseTurn::get_r2(frame, NoiseTurn::FRAMES);

    Self {
      noise: Vec4::new(x, y, 0.0, 0.0),
      radius: LAMP_RADIUS,
      _pad: [0.0; 3],
    }
  }
}
