use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::pass::noise_turn::NoiseTurn;

/// What the lamps' traced shadows read, as `shaders/frame/light_tracing.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "LightTrace")]
pub struct LightTraceUniform {
  /// What this frame turns each pixel's noise by, each way, then nothing.
  pub noise: Vec4,
}

impl LightTraceUniform {
  /// On the noise's `frame`.
  pub fn new(frame: u32) -> Self {
    let [x, y] = NoiseTurn::get_r2(frame, NoiseTurn::FRAMES);

    Self {
      noise: Vec4::new(x, y, 0.0, 0.0),
    }
  }
}
