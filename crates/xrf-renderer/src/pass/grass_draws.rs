/// What one frame's grass draws from: the model arenas, the draw arguments the planting wrote, and how many models.
pub struct GrassDraws<'a> {
  pub positions: &'a wgpu::Buffer,
  pub uvs: &'a wgpu::Buffer,
  pub indices: &'a wgpu::Buffer,
  pub args: &'a wgpu::Buffer,
  pub models: u32,
}
