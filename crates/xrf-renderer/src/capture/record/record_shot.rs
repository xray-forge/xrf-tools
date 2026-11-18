use std::sync::Arc;

use crate::capture::record::record_layout::RecordLayout;
use crate::frame::readback::gpu_readback::GpuReadback;

/// A recording frame drawn this frame: the viewport's finished frame at the recording's size, copied to a ring buffer.
pub(crate) struct RecordShot {
  /// Its buffer's index in the ring.
  pub(crate) slot: usize,
  readback: Arc<GpuReadback>,
  layout: RecordLayout,
}

impl RecordShot {
  /// Recording frame format: BGRA, as Media Foundation's RGB32 takes it.
  pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

  pub(crate) fn new(slot: usize, readback: Arc<GpuReadback>, layout: RecordLayout) -> Self {
    Self { slot, readback, layout }
  }

  /// The recording's size in pixels.
  pub(crate) fn get_size(&self) -> (u32, u32) {
    self.layout.size
  }

  /// Encodes the copy out of `texture`, which must be at the recording's size; requested back after submission.
  pub(crate) fn encode(&self, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::Texture) {
    let (width, height): (u32, u32) = self.layout.size;

    encoder.copy_texture_to_buffer(
      texture.as_image_copy(),
      wgpu::TexelCopyBufferInfo {
        buffer: self.readback.get_buffer(),
        layout: wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(self.layout.row),
          rows_per_image: Some(height),
        },
      },
      wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
    );
    self.readback.mark_recorded();
  }
}
