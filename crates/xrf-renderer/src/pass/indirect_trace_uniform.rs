use glam::Vec2;
use xrf_renderer_core::ShaderStruct;

use crate::pass::noise_turn::NoiseTurn;

/// Metres a bounce's ray reaches at most: past them the light it would carry has fallen off.
const REACH: f32 = 30.0;

/// What the traced indirect light reads, as `shaders/frame/indirect_trace.wgsl` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "IndirectTrace")]
pub struct IndirectTraceUniform {
  /// What this frame turns each pixel's noise by, each way.
  pub noise: Vec2,
  /// The metres a ray reaches.
  pub reach: f32,
  /// The light's intensity.
  pub intensity: f32,
}

impl IndirectTraceUniform {
  /// On the noise's `frame`, at the indirect light's `intensity`.
  pub fn new(frame: u32, intensity: f32) -> Self {
    let [x, y] = NoiseTurn::get_r2(frame, NoiseTurn::FRAMES);

    Self {
      noise: Vec2::new(x, y),
      reach: REACH,
      intensity,
    }
  }
}
