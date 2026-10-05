use glam::Vec2;
use xrf_renderer_core::ShaderStruct;

/// What the upscale passes read, as WGSL's `Upscale`: the frame's size at the viewport, and RCAS's sharpness.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Upscale")]
pub struct UpscaleUniform {
  pub output_size: Vec2,
  pub sharpness: f32,
  pub _pad: f32,
}
