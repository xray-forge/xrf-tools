use crate::frame::gpu_readback::GpuReadback;

/// Reads a few words the GPU counted back without ever waiting on it: a copy goes out with a frame, is mapped once that
/// frame's work is done, and is read a frame or more later, while the next copy waits its turn.
pub struct StatsReadback {
  readback: GpuReadback,
  values: [u32; 4],
}

impl StatsReadback {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      readback: GpuReadback::new(device, "stats readback", 16),
      values: [0; 4],
    }
  }

  /// Copies the counts out with this frame's work, unless an earlier copy is still on its way.
  pub fn record(&self, encoder: &mut wgpu::CommandEncoder, source: &wgpu::Buffer, offset: u64) {
    if self.readback.is_free() {
      encoder.copy_buffer_to_buffer(source, offset, self.readback.get_buffer(), 0, 16);
      self.readback.mark_recorded();
    }
  }

  /// Asks for the copy recorded with the frame just submitted.
  pub fn request(&self) {
    self.readback.request();
  }

  /// The latest counts read back, refreshed once a mapped copy is there.
  pub fn take(&mut self) -> [u32; 4] {
    if let Some(Ok(values)) = self
      .readback
      .take(|bytes| bytemuck::pod_read_unaligned::<[u32; 4]>(&bytes[..16]))
    {
      self.values = values;
    }

    self.values
  }
}
