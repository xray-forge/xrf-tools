use std::sync::Arc;

use crate::capture::record::record_layout::RecordLayout;
use crate::frame::readback::gpu_readback::GpuReadback;

/// A recording's frame this frame draws: the viewport's finished frame presented at the recording's size into a texture
/// of its own and copied into a buffer of the recording's ring.
pub(crate) struct RecordShot {
  /// Its buffer's place in the ring, and the buffer.
  pub(crate) slot: usize,
  readback: Arc<GpuReadback>,
  layout: RecordLayout,
}

impl RecordShot {
  /// The format a recording's frames are drawn in: blue first, as Media Foundation's RGB32 takes them.
  pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

  pub(crate) fn new(slot: usize, readback: Arc<GpuReadback>, layout: RecordLayout) -> Self {
    Self { slot, readback, layout }
  }

  /// Pixels across and down, the recording's.
  pub(crate) fn get_size(&self) -> (u32, u32) {
    self.layout.size
  }

  /// Copies the frame out of `texture`, presented at its size; asked for once the copy is submitted.
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
