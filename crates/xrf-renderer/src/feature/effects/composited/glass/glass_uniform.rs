use xrf_renderer_core::ShaderStruct;

/// What the enhanced glass reads as its `Glass`: how far it bends what is behind it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Glass")]
pub struct GlassUniform {
  pub refraction: f32,
}
