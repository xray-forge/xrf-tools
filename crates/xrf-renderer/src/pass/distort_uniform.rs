use xrf_renderer_core::ShaderStruct;

/// What the distortion before the resolve reads as its `Distort`: how far the distortion target moves the frame, a
/// share of the screen (`def_distort`), and one where a bloom draws, which its `z` blurs the frame towards.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Distort")]
pub struct DistortUniform {
  pub strength: f32,
  pub is_blurred: u32,
}
