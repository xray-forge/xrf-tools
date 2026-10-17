use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

use crate::pass::noise_turn::NoiseTurn;

/// Metres a bounce's ray reaches at most: past them the light it would carry has fallen off.
const REACH: f32 = 30.0;

/// What the traced indirect light reads, as `shaders/frame/indirect_trace.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "IndirectTrace")]
pub struct IndirectTraceUniform {
  /// What this frame turns each pixel's noise by, each way; the metres a ray reaches; the light's intensity.
  pub noise: Vec4,
}

impl IndirectTraceUniform {
  /// On the noise's `frame`, at the indirect light's `intensity`.
  pub fn new(frame: u32, intensity: f32) -> Self {
    let [x, y] = NoiseTurn::get_r2(frame, NoiseTurn::FRAMES);

    Self {
      noise: Vec4::new(x, y, REACH, intensity),
    }
  }
}
