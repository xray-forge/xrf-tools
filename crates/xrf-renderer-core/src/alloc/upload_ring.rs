use crate::alloc::upload_slice::UploadSlice;

/// One frame's uniforms and per-pass data packed into one buffer and uploaded in one write, each value bound at its
/// dynamic offset. Values are pushed during the frame and valid from `flush` until the next frame's.
pub struct UploadRing {
  label: &'static str,
  usage: wgpu::BufferUsages,
  alignment: u32,
  buffer: wgpu::Buffer,
  staging: Vec<u8>,
  generation: u64,
}

impl UploadRing {
  /// A ring of `capacity` bytes to start with, aligned for both uniform and storage dynamic offsets; `usage` gains
  /// the copy its write needs.
  pub fn new(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages, capacity: u64) -> Self {
    let limits: wgpu::Limits = device.limits();
    let usage: wgpu::BufferUsages = usage | wgpu::BufferUsages::COPY_DST;

    Self {
      buffer: Self::create_buffer(device, label, usage, capacity.max(256)),
      label,
      usage,
      alignment: limits
        .min_uniform_buffer_offset_alignment
        .max(limits.min_storage_buffer_offset_alignment),
      staging: Vec::new(),
      generation: 0,
    }
  }

  pub fn get_buffer(&self) -> &wgpu::Buffer {
    &self.buffer
  }

  /// Changes each time `flush` replaces the buffer with a larger one, after which anything bound to it is stale.
  pub fn get_generation(&self) -> u64 {
    self.generation
  }

  /// The offset every pushed value starts at a multiple of.
  pub fn get_alignment(&self) -> u32 {
    self.alignment
  }

  /// Bytes pushed this frame, padding included.
  pub fn get_pushed(&self) -> u64 {
    self.staging.len() as u64
  }

  pub fn push<T: bytemuck::Pod>(&mut self, value: &T) -> UploadSlice {
    self.push_bytes(bytemuck::bytes_of(value))
  }

  pub fn push_bytes(&mut self, bytes: &[u8]) -> UploadSlice {
    let offset: usize = self.staging.len().next_multiple_of(self.alignment as usize);

    self.staging.resize(offset, 0);
    self.staging.extend_from_slice(bytes);
    // A write moves whole four-byte words.
    self.staging.resize(
      self
        .staging
        .len()
        .next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT as usize),
      0,
    );

    UploadSlice {
      offset: offset as u32,
      size: bytes.len() as u32,
    }
  }

  /// Uploads what this frame pushed in one write, first replacing the buffer with one large enough, and starts the next
  /// frame's pushing.
  pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    if self.staging.len() as u64 > self.buffer.size() {
      let capacity: u64 = (self.staging.len() as u64).next_power_of_two();

      self.buffer = Self::create_buffer(device, self.label, self.usage, capacity);
      self.generation += 1;
    }

    if !self.staging.is_empty() {
      queue.write_buffer(&self.buffer, 0, &self.staging);
    }

    self.staging.clear();
  }

  fn create_buffer(
    device: &wgpu::Device,
    label: &'static str,
    usage: wgpu::BufferUsages,
    capacity: u64,
  ) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(label),
      size: capacity,
      usage,
      mapped_at_creation: false,
    })
  }
}
