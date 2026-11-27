/// A texture a layer samples, on the GPU: the texture, the bind group the layer pass binds it by with its sampler, and
/// the size and sampling it was made for.
pub(crate) struct LayerTexture {
  pub texture: wgpu::Texture,
  pub bind_group: wgpu::BindGroup,
  pub width: u32,
  pub height: u32,
  pub is_nearest: bool,
}
