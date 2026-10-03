/// What a viewport's rain draw binds: its uniform, the cover, both textures and the splash's model.
pub struct RainBindings<'a> {
  pub uniform: &'a wgpu::Buffer,
  pub cover: &'a wgpu::TextureView,
  pub streak: &'a wgpu::TextureView,
  pub splash: &'a wgpu::TextureView,
  pub vertices: &'a wgpu::Buffer,
  pub indices: &'a wgpu::Buffer,
}
