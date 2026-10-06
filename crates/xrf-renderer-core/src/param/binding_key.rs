/// What one binding of a cached bind group binds, by the resource's identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BindingKey {
  Buffer(wgpu::Buffer, Option<u64>),
  TextureView(wgpu::TextureView),
  Sampler(wgpu::Sampler),
}
