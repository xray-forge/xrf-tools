/// What makes two transient buffers interchangeable: their size and the usage the graph gathered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransientBufferKey {
  pub size: u64,
  pub usage: wgpu::BufferUsages,
}
