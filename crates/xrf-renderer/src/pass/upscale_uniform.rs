use glam::Vec2;
use xrf_renderer_core::ShaderStruct;

/// What the upscale passes read, as WGSL's `Upscale`: the frame's size at the viewport, RCAS's sharpness, and one where
/// the frame holds linear light, which both read through its reversible tonemap.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Upscale")]
pub struct UpscaleUniform {
  pub output_size: Vec2,
  pub sharpness: f32,
  pub linear: f32,
}
