/// What one binding of a pass's parameters binds this frame.
#[derive(Clone, Copy, Debug)]
pub enum PassBinding<'r> {
  /// A whole buffer, or the first `size` bytes of one bound at a dynamic offset.
  Buffer(&'r wgpu::Buffer),
  BufferRange {
    buffer: &'r wgpu::Buffer,
    size: u64,
  },
  TextureView(&'r wgpu::TextureView),
  Sampler(&'r wgpu::Sampler),
}
