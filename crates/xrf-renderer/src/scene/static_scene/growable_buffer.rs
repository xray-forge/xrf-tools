use crate::contract::render_pool_use::RenderPoolUse;

/// A storage buffer appended to as a scene loads: it doubles and copies what it holds when an append outgrows it, so
/// what the GPU already reads stays where it is.
pub struct GrowableBuffer {
  label: &'static str,
  usage: wgpu::BufferUsages,
  buffer: wgpu::Buffer,
  length: u64,
  /// Bumped each time the buffer is replaced, so whatever binds it knows to bind it again.
  generation: u64,
}

impl GrowableBuffer {
  /// Bytes a buffer starts at, so a small scene allocates little and a large one doubles few times.
  const INITIAL: u64 = 64 * 1024;

  pub fn new(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages) -> Self {
    let usage: wgpu::BufferUsages = usage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;

    Self {
      buffer: Self::create(device, label, usage, Self::INITIAL),
      label,
      usage,
      length: 0,
      generation: 0,
    }
  }

  pub fn get_buffer(&self) -> &wgpu::Buffer {
    &self.buffer
  }

  pub fn get_generation(&self) -> u64 {
    self.generation
  }

  /// Bytes the buffer holds on the GPU, written or not.
  pub fn get_capacity(&self) -> u64 {
    self.buffer.size()
  }

  /// Bytes written so far.
  pub fn get_length(&self) -> u64 {
    self.length
  }

  /// Records of `stride` bytes written so far, and how many the buffer holds before it grows.
  pub fn get_use(&self, stride: usize) -> RenderPoolUse {
    let stride: u64 = stride.max(1) as u64;

    RenderPoolUse {
      used: (self.length / stride) as u32,
      capacity: (self.buffer.size() / stride) as u32,
    }
  }

  /// Writes bytes after what the buffer holds, growing it first where they do not fit; answers where they start.
  pub fn append(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    bytes: &[u8],
  ) -> u64 {
    let start: u64 = self.length;
    let end: u64 = start + bytes.len() as u64;

    self.reserve(device, encoder, end);

    if !bytes.is_empty() {
      queue.write_buffer(&self.buffer, start, bytes);
    }

    self.length = end;

    start
  }

  /// Makes the buffer at least `size` bytes, keeping what it holds; it replaces nothing when it already is.
  pub fn reserve(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, size: u64) {
    if size <= self.buffer.size() {
      return;
    }

    let capacity: u64 = size
      .max(self.buffer.size() * 2)
      .next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT * 64);
    let grown: wgpu::Buffer = Self::create(device, self.label, self.usage, capacity);

    if self.length > 0 {
      encoder.copy_buffer_to_buffer(&self.buffer, 0, &grown, 0, self.length.next_multiple_of(4));
    }

    self.buffer = grown;
    self.generation += 1;
  }

  fn create(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(label),
      size,
      usage,
      mapped_at_creation: false,
    })
  }
}
