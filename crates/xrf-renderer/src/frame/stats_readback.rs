use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// Nothing copied, a copy recorded but not submitted, a map asked for, a map done.
const IDLE: u8 = 0;
const RECORDED: u8 = 1;
const MAPPING: u8 = 2;
const MAPPED: u8 = 3;

/// Reads a few words the GPU counted back without ever waiting on it: a copy goes out with a frame, is mapped once that
/// frame's work is done, and is read a frame or more later, while the next copy waits its turn.
pub struct StatsReadback {
  buffer: wgpu::Buffer,
  state: Arc<AtomicU8>,
  values: [u32; 4],
}

impl StatsReadback {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      buffer: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("stats readback"),
        size: 16,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
      }),
      state: Arc::new(AtomicU8::new(IDLE)),
      values: [0; 4],
    }
  }

  /// Copies the counts out with this frame's work, unless an earlier copy is still on its way.
  pub fn record(&self, encoder: &mut wgpu::CommandEncoder, source: &wgpu::Buffer, offset: u64) {
    if self.state.load(Ordering::Acquire) == IDLE {
      encoder.copy_buffer_to_buffer(source, offset, &self.buffer, 0, 16);
      self.state.store(RECORDED, Ordering::Release);
    }
  }

  /// Asks for the copy recorded with the frame just submitted.
  pub fn request(&self) {
    if self.state.load(Ordering::Acquire) == RECORDED {
      let state: Arc<AtomicU8> = Arc::clone(&self.state);

      self.state.store(MAPPING, Ordering::Release);
      self.buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
        state.store(if result.is_ok() { MAPPED } else { IDLE }, Ordering::Release);
      });
    }
  }

  /// The latest counts read back, refreshed once a mapped copy is there.
  pub fn take(&mut self) -> [u32; 4] {
    if self.state.load(Ordering::Acquire) == MAPPED {
      if let Ok(view) = self.buffer.slice(..).get_mapped_range() {
        self.values = bytemuck::pod_read_unaligned::<[u32; 4]>(&view[..16]);
      }

      self.buffer.unmap();
      self.state.store(IDLE, Ordering::Release);
    }

    self.values
  }
}
