/// A buffer the pool keeps between frames, and the frame that last used it.
pub(crate) struct PooledBuffer {
  pub buffer: wgpu::Buffer,
  pub last_frame: u64,
}
