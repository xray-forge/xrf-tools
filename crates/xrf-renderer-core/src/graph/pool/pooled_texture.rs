/// A texture the pool keeps between frames, with its whole view and the frame that last drew with it.
pub(crate) struct PooledTexture {
  pub texture: wgpu::Texture,
  pub view: wgpu::TextureView,
  pub last_frame: u64,
}
