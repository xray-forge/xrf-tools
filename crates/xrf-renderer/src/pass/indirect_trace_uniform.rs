use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// Metres a bounce's ray reaches at most: past them the light it would carry has fallen off.
const REACH: f32 = 30.0;

/// The R2 sequence's steps, which turn each pixel's ray from one frame to the next.
const NOISE_TURN: [f32; 2] = [0.754_877_7, 0.569_840_3];

/// Frames the noise cycles through before it repeats.
const NOISE_FRAMES: u32 = 64;

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
    let turn: f32 = (frame % NOISE_FRAMES) as f32;

    Self {
      noise: Vec4::new(
        (turn * NOISE_TURN[0]).fract(),
        (turn * NOISE_TURN[1]).fract(),
        REACH,
        intensity,
      ),
    }
  }
}
