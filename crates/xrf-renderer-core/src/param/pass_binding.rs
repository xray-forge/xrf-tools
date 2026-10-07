/// What one binding of a pass's parameters binds this frame.
#[derive(Clone, Debug)]
pub enum PassBinding<'r> {
  /// A whole buffer, or the first `size` bytes of one bound at a dynamic offset.
  Buffer(&'r wgpu::Buffer),
  BufferRange {
    buffer: &'r wgpu::Buffer,
    offset: u64,
    size: u64,
  },
  TextureView(&'r wgpu::TextureView),
  /// A binding array's views, one an element.
  TextureViewArray(Vec<&'r wgpu::TextureView>),
  Sampler(&'r wgpu::Sampler),
}
