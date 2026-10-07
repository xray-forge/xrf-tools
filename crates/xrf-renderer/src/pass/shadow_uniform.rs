use glam::{Mat4, Vec4};
use xrf_renderer_core::ShaderStruct;

/// The sun's cascades as `shaders/common/sun_shadow.wgsl` declares them: each map's matrix as it was drawn, its texel, and how
/// the lookup filters, offsets and blends.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Shadows")]
pub struct ShadowUniform {
  /// Renderer space into each cascade's clip space, as its map was last drawn.
  pub matrices: [Mat4; 4],
  /// Metres one texel of each cascade's map is across.
  pub texels: Vec4,
  /// xyz: where the camera looks, which the last cascade fades out towards.
  pub forward: Vec4,
  pub count: u32,
  pub filter_reach: u32,
  pub resolution: f32,
  pub bias: f32,
  pub blend: f32,
  pub _pad: [f32; 3],
}

impl Default for ShadowUniform {
  fn default() -> Self {
    Self {
      matrices: [Mat4::IDENTITY; 4],
      texels: Vec4::ZERO,
      forward: Vec4::ZERO,
      count: 0,
      filter_reach: 0,
      resolution: 1.0,
      bias: 0.0,
      blend: 0.0,
      _pad: [0.0; 3],
    }
  }
}
