/// What one binding of a cached bind group binds, by the resource's identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BindingKey {
  /// A buffer, with the offset and size of the range bound where it binds one.
  Buffer(wgpu::Buffer, Option<(u64, u64)>),
  TextureView(wgpu::TextureView),
  Sampler(wgpu::Sampler),
}
