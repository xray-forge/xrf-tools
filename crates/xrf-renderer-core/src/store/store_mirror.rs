use crate::alloc::SpanWrites;
use crate::store::proxy_store::ProxyStore;

/// A store's records on the GPU, element for element: each sync uploads what was added, changed or moved since the
/// last as few contiguous writes, growing the buffer by a copy into one twice the size when the store outgrew it.
pub struct StoreMirror {
  label: &'static str,
  stride: u64,
  usage: wgpu::BufferUsages,
  buffer: wgpu::Buffer,
  /// Records the buffer holds room for.
  capacity: u32,
  /// Records the store held at the last sync.
  count: u32,
  writes: SpanWrites,
  generation: u64,
}

impl StoreMirror {
  /// Records a buffer holds at the least, so a small store neither starts empty nor grows a record at a time.
  const MINIMUM: u32 = 64;

  /// A mirror of records of `stride` bytes; `usage` gains the copies syncing and growing need.
  ///
  /// # Panics
  ///
  /// When `stride` is not a whole number of the four bytes a copy moves.
  pub fn new(device: &wgpu::Device, label: &'static str, stride: u64, usage: wgpu::BufferUsages) -> Self {
    assert!(
      stride > 0 && stride.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT),
      "Store mirror '{label}' has a stride of {stride} bytes, not a multiple of {}",
      wgpu::COPY_BUFFER_ALIGNMENT
    );

    let usage: wgpu::BufferUsages = usage | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST;

    Self {
      buffer: Self::create_buffer(device, label, stride, usage, Self::MINIMUM),
      label,
      stride,
      usage,
      capacity: Self::MINIMUM,
      count: 0,
      writes: SpanWrites::default(),
      generation: 0,
    }
  }

  pub fn get_buffer(&self) -> &wgpu::Buffer {
    &self.buffer
  }

  /// Records the buffer held at the last sync, which is what a pass reading it should read.
  pub fn get_count(&self) -> u32 {
    self.count
  }

  /// Records the buffer holds room for before it grows.
  pub fn get_capacity(&self) -> u32 {
    self.capacity
  }

  /// Bytes the buffer holds on the GPU.
  pub fn get_bytes(&self) -> u64 {
    self.buffer.size()
  }

  /// Changes each time the buffer is replaced by a larger one, after which anything bound to the old one is stale.
  pub fn get_generation(&self) -> u64 {
    self.generation
  }

  /// Uploads the store's records added, changed or moved since the last sync, each as `to_record` writes it.
  ///
  /// # Panics
  ///
  /// When a record is not `stride` bytes.
  pub fn sync<T, R: bytemuck::Pod>(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    store: &mut ProxyStore<T>,
    to_record: impl Fn(&T) -> R,
  ) {
    assert_eq!(
      size_of::<R>() as u64,
      self.stride,
      "Store mirror '{}' holds records of {} bytes",
      self.label,
      self.stride
    );

    let count: u32 = store.len() as u32;

    if count > self.capacity {
      self.grow(device, queue, count);
    }

    for dense in store.take_changed() {
      let record: R = to_record(&store.as_slice()[dense as usize]);

      self
        .writes
        .push(u64::from(dense) * self.stride, bytemuck::bytes_of(&record));
    }

    for (offset, bytes) in self.writes.take_runs() {
      queue.write_buffer(&self.buffer, offset, &bytes);
    }

    self.count = count;
  }

  fn grow(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, count: u32) {
    let capacity: u32 = count.next_power_of_two().max(self.capacity * 2);
    let grown: wgpu::Buffer = Self::create_buffer(device, self.label, self.stride, self.usage, capacity);
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some(self.label),
    });

    encoder.copy_buffer_to_buffer(&self.buffer, 0, &grown, 0, self.buffer.size());
    queue.submit([encoder.finish()]);
    self.buffer = grown;
    self.capacity = capacity;
    self.generation += 1;
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
