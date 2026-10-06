use xrf_renderer_core::ShaderStruct;

/// The head of the exposure's state, as the passes tonemapping with it read it: the scale, adapted frame by frame.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Exposure")]
pub struct ExposureHead {
  pub adapted: f32,
}
