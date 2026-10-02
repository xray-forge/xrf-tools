/// What one viewport's local lights are bound by: their records, the clusters' counts and lists, and the uniform.
pub struct LightBuffers<'a> {
  pub records: &'a wgpu::Buffer,
  pub counts: &'a wgpu::Buffer,
  pub items: &'a wgpu::Buffer,
  pub uniform: &'a wgpu::Buffer,
}
