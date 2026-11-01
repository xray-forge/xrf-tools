/// The local lights' shadow atlas as a pass reads it: its view, and the sampler comparing a depth with it between four
/// texels.
#[derive(Clone, Copy)]
pub struct LightShadowAtlas<'a> {
  pub view: &'a wgpu::TextureView,
  pub sampler: &'a wgpu::Sampler,
}
