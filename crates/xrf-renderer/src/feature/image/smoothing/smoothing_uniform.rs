use xrf_renderer_core::ShaderStruct;

/// What FXAA and SMAA read as their `Smoothing`: one where the frame holds linear light, whose edges they find over its
/// reversibly tonemapped colour.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Smoothing")]
pub struct SmoothingUniform {
  pub linear: f32,
}
