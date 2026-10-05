use xrf_error::{XrfError, XrfResult};

use crate::alloc::span::Span;
use crate::alloc::span_allocator::SpanAllocator;
use crate::alloc::span_writes::SpanWrites;

/// A GPU buffer of fixed-size elements that ranges are taken from and given back to, so a scene's contents can be added
/// and removed. Growing copies it into one twice the size; a held range keeps its offset. Writes wait for `flush`,
/// which uploads them as few contiguous runs.
pub struct SpanBuffer {
  label: &'static str,
  stride: u64,
  usage: wgpu::BufferUsages,
  buffer: wgpu::Buffer,
  allocator: SpanAllocator,
  writes: SpanWrites,
  generation: u64,
}

impl SpanBuffer {
  /// A buffer of `capacity` elements of `stride` bytes; `usage` gains the copies growing needs.
  ///
  /// # Panics
  ///
  /// When `stride` is not a whole number of the four bytes a copy moves.
  pub fn new(
    device: &wgpu::Device,
    label: &'static str,
    stride: u64,
    usage: wgpu::BufferUsages,
    capacity: u32,
  ) -> Self {
    assert!(
      stride > 0 && stride.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT),
      "Span buffer '{label}' has a stride of {stride} bytes, not a multiple of {}",
      wgpu::COPY_BUFFER_ALIGNMENT
    );

    let usage: wgpu::BufferUsages = usage | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST;
    let allocator: SpanAllocator = SpanAllocator::new(capacity);

    Self {
      buffer: Self::create_buffer(device, label, stride, usage, allocator.get_capacity()),
      label,
      stride,
      usage,
      allocator,
      writes: SpanWrites::default(),
      generation: 0,
    }
  }

  pub fn get_buffer(&self) -> &wgpu::Buffer {
    &self.buffer
  }

  /// Changes each time the buffer is replaced by a larger one, after which anything bound to the old one is stale.
  pub fn get_generation(&self) -> u64 {
    self.generation
  }

  pub fn get_capacity(&self) -> u32 {
    self.allocator.get_capacity()
  }

  pub fn get_free(&self) -> u32 {
    self.allocator.get_free()
  }

  /// A range of `count` elements, growing the buffer when no region has room. Growing submits its copy at once; writes
  /// still queued go to the grown buffer at the next `flush`, after the copy.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffer would outgrow what the device allows.
  pub fn allocate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, count: u32) -> XrfResult<Span> {
    if let Some(span) = self.allocator.allocate(count) {
      return Ok(span);
    }

    let capacity: u32 = self.allocator.get_grown_capacity(count);
    let bytes: u64 = u64::from(capacity) * self.stride;

    if bytes > device.limits().max_buffer_size {
      return Err(XrfError::new_invalid_error(format!(
        "Span buffer '{}' would grow to {bytes} bytes, past the device's {}",
        self.label,
        device.limits().max_buffer_size
      )));
    }

    let grown: wgpu::Buffer = Self::create_buffer(device, self.label, self.stride, self.usage, capacity);
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some(self.label),
    });

    encoder.copy_buffer_to_buffer(&self.buffer, 0, &grown, 0, self.buffer.size());
    queue.submit([encoder.finish()]);

    self.buffer = grown;
    self.allocator.grow(capacity);
    self.generation += 1;

    self.allocator.allocate(count).ok_or_else(|| {
      XrfError::new_unexpected_error(format!(
        "Span buffer '{}' has no room for {count} after growing",
        self.label
      ))
    })
  }

  pub fn free(&mut self, span: Span) {
    self.allocator.free(span);
  }

  /// Queues `bytes` for the span's elements from its `first`, written at the next `flush`.
  ///
  /// # Panics
  ///
  /// When the bytes reach past the span or do not fill whole elements.
  pub fn write(&mut self, span: &Span, first: u32, bytes: &[u8]) {
    let elements: u64 = bytes.len() as u64 / self.stride;

    assert!(
      (bytes.len() as u64).is_multiple_of(self.stride) && u64::from(first) + elements <= u64::from(span.count),
      "A write of {} bytes from element {first} does not fit span buffer '{}''s span of {} elements of {} bytes",
      bytes.len(),
      self.label,
      span.count,
      self.stride
    );

    self
      .writes
      .push((u64::from(span.offset) + u64::from(first)) * self.stride, bytes);
  }

  /// Uploads the queued writes, a run of contiguous bytes at a time.
  pub fn flush(&mut self, queue: &wgpu::Queue) {
    for (offset, bytes) in self.writes.take_runs() {
      queue.write_buffer(&self.buffer, offset, &bytes);
    }
  }

  fn create_buffer(
    device: &wgpu::Device,
    label: &'static str,
    stride: u64,
    usage: wgpu::BufferUsages,
    capacity: u32,
  ) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(label),
      size: u64::from(capacity) * stride,
      usage,
      mapped_at_creation: false,
    })
  }
}
