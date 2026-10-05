use xrf_error::{XrfError, XrfResult};

use crate::alloc::span::Span;
use crate::alloc::span_allocator::SpanAllocator;
use crate::alloc::span_lane::SpanLane;
use crate::alloc::span_writes::SpanWrites;

/// GPU buffers of fixed-size elements that ranges are taken from and given back to, so a scene's contents can be added
/// and removed. Its lanes are parallel arrays sharing one allocator, so a range addresses the same elements of each.
/// Growing copies each lane into one twice the size; a held range keeps its offset. Writes wait for `flush`, which
/// uploads them as few contiguous runs.
pub struct SpanBuffer {
  usage: wgpu::BufferUsages,
  lanes: Vec<LaneBuffer>,
  allocator: SpanAllocator,
  generation: u64,
}

/// One lane's buffer and the writes waiting for it.
struct LaneBuffer {
  lane: SpanLane,
  buffer: wgpu::Buffer,
  writes: SpanWrites,
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
    Self::with_lanes(device, &[SpanLane::new(label, stride)], usage, capacity)
  }

  /// Parallel buffers of `capacity` elements, one a lane, each element of the size its lane gives.
  ///
  /// # Panics
  ///
  /// When there is no lane, or a lane's stride is not a whole number of the four bytes a copy moves.
  pub fn with_lanes(device: &wgpu::Device, lanes: &[SpanLane], usage: wgpu::BufferUsages, capacity: u32) -> Self {
    assert!(!lanes.is_empty(), "A span buffer holds a lane at least");

    for lane in lanes {
      assert!(
        lane.stride > 0 && lane.stride.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT),
        "Span buffer '{}' has a stride of {} bytes, not a multiple of {}",
        lane.label,
        lane.stride,
        wgpu::COPY_BUFFER_ALIGNMENT
      );
    }

    let usage: wgpu::BufferUsages = usage | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST;
    let allocator: SpanAllocator = SpanAllocator::new(capacity);

    Self {
      lanes: lanes
        .iter()
        .map(|lane| LaneBuffer {
          lane: *lane,
          buffer: create_buffer(device, lane, usage, allocator.get_capacity()),
          writes: SpanWrites::default(),
        })
        .collect(),
      usage,
      allocator,
      generation: 0,
    }
  }

  /// The first lane's buffer.
  pub fn get_buffer(&self) -> &wgpu::Buffer {
    self.get_lane(0)
  }

  /// A lane's buffer, by its place in the lanes it was made with.
  pub fn get_lane(&self, lane: usize) -> &wgpu::Buffer {
    &self.lanes[lane].buffer
  }

  /// Changes each time the buffers are replaced by larger ones, after which anything bound to the old ones is stale.
  pub fn get_generation(&self) -> u64 {
    self.generation
  }

  pub fn get_capacity(&self) -> u32 {
    self.allocator.get_capacity()
  }

  pub fn get_free(&self) -> u32 {
    self.allocator.get_free()
  }

  /// Bytes its lanes hold on the GPU, held or free.
  pub fn get_bytes(&self) -> u64 {
    self.lanes.iter().map(|lane| lane.buffer.size()).sum()
  }

  /// A range of `count` elements, growing the buffers when no region has room. Growing submits its copies at once;
  /// writes still queued go to the grown buffers at the next `flush`, after the copies.
  ///
  /// # Errors
  ///
  /// Returns an error when a lane would outgrow what the device allows.
  pub fn allocate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, count: u32) -> XrfResult<Span> {
    if let Some(span) = self.allocator.allocate(count) {
      return Ok(span);
    }

    let capacity: u32 = self.allocator.get_grown_capacity(count);

    for lane in &self.lanes {
      let bytes: u64 = u64::from(capacity) * lane.lane.stride;

      if bytes > device.limits().max_buffer_size {
        return Err(XrfError::new_invalid_error(format!(
          "Span buffer '{}' would grow to {bytes} bytes, past the device's {}",
          lane.lane.label,
          device.limits().max_buffer_size
        )));
      }
    }

    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some(self.lanes[0].lane.label),
    });

    for lane in &mut self.lanes {
      let grown: wgpu::Buffer = create_buffer(device, &lane.lane, self.usage, capacity);

      encoder.copy_buffer_to_buffer(&lane.buffer, 0, &grown, 0, lane.buffer.size());
      lane.buffer = grown;
    }

    queue.submit([encoder.finish()]);
    self.allocator.grow(capacity);
    self.generation += 1;

    self.allocator.allocate(count).ok_or_else(|| {
      XrfError::new_unexpected_error(format!(
        "Span buffer '{}' has no room for {count} after growing",
        self.lanes[0].lane.label
      ))
    })
  }

  pub fn free(&mut self, span: Span) {
    self.allocator.free(span);
  }

  /// Queues `bytes` for the first lane's elements of the span from its `first`, written at the next `flush`.
  ///
  /// # Panics
  ///
  /// When the bytes reach past the span or do not fill whole elements.
  pub fn write(&mut self, span: &Span, first: u32, bytes: &[u8]) {
    self.write_lane(span, 0, first, bytes);
  }

  /// Queues `bytes` for a lane's elements of the span from its `first`, written at the next `flush`.
  ///
  /// # Panics
  ///
  /// When the bytes reach past the span or do not fill whole elements.
  pub fn write_lane(&mut self, span: &Span, lane: usize, first: u32, bytes: &[u8]) {
    let lane: &mut LaneBuffer = &mut self.lanes[lane];
    let stride: u64 = lane.lane.stride;
    let elements: u64 = bytes.len() as u64 / stride;

    assert!(
      (bytes.len() as u64).is_multiple_of(stride) && u64::from(first) + elements <= u64::from(span.count),
      "A write of {} bytes from element {first} does not fit span buffer '{}''s span of {} elements of {stride} bytes",
      bytes.len(),
      lane.lane.label,
      span.count,
    );

    lane
      .writes
      .push((u64::from(span.offset) + u64::from(first)) * stride, bytes);
  }

  /// Queues zeros over every lane's elements of the span, so what a shader walking the whole buffer meets there reads
  /// as nothing once the span is freed.
  pub fn clear(&mut self, span: &Span) {
    for lane in &mut self.lanes {
      let zeros: Vec<u8> = vec![0; (u64::from(span.count) * lane.lane.stride) as usize];

      lane.writes.push(u64::from(span.offset) * lane.lane.stride, &zeros);
    }
  }

  /// Uploads the queued writes, a run of contiguous bytes at a time.
  pub fn flush(&mut self, queue: &wgpu::Queue) {
    for lane in &mut self.lanes {
      for (offset, bytes) in lane.writes.take_runs() {
        queue.write_buffer(&lane.buffer, offset, &bytes);
      }
    }
  }
}

fn create_buffer(device: &wgpu::Device, lane: &SpanLane, usage: wgpu::BufferUsages, capacity: u32) -> wgpu::Buffer {
  device.create_buffer(&wgpu::BufferDescriptor {
    label: Some(lane.label),
    size: u64::from(capacity) * lane.stride,
    usage,
    mapped_at_creation: false,
  })
}
