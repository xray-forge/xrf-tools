use xrf_renderer_core::ShaderStruct;

/// The head of the exposure's state, as the passes tonemapping with it read it: the scale, adapted frame by frame, and
/// the scale the last frame's tonemap used, which a read of that frame's colour inverts by.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Exposure")]
pub struct ExposureHead {
  pub adapted: f32,
  pub used: f32,
}
