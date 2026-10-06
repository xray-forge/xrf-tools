use xrf_error::XrfResult;

use crate::frame::gpu_readback::GpuReadback;

/// A pick's one texel and its depth, and the buffers it is read back through: a few, so a pick asked while another is
/// on its way is drawn without waiting for it.
pub struct PickTarget {
  pub color: wgpu::TextureView,
  pub depth: wgpu::TextureView,
  texture: wgpu::Texture,
  readbacks: [GpuReadback; Self::READBACKS],
}

impl PickTarget {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Uint;
  /// Picks on their way back at once.
  pub const READBACKS: usize = 3;

  pub fn new(device: &wgpu::Device) -> Self {
    let create = |format: wgpu::TextureFormat, usage: wgpu::TextureUsages| -> wgpu::Texture {
      device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pick"),
        size: wgpu::Extent3d {
          width: 1,
          height: 1,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
      })
    };
    let texture: wgpu::Texture = create(
      Self::FORMAT,
      wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    );
    let depth: wgpu::Texture = create(
      wgpu::TextureFormat::Depth32Float,
      wgpu::TextureUsages::RENDER_ATTACHMENT,
    );

    Self {
      color: texture.create_view(&Default::default()),
      depth: depth.create_view(&Default::default()),
      readbacks: std::array::from_fn(|_| {
        GpuReadback::new(device, "pick readback", wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64)
      }),
      texture,
    }
  }

  /// Whether a pick may be drawn this frame: a readback is free for it.
  pub fn has_free_readback(&self) -> bool {
    self.readbacks.iter().any(GpuReadback::is_free)
  }

  /// Copies the texel out with the frame's work into a free readback, and answers which.
  pub fn copy_out(&self, encoder: &mut wgpu::CommandEncoder) -> Option<usize> {
    let slot: usize = self.readbacks.iter().position(GpuReadback::is_free)?;
    let readback: &GpuReadback = &self.readbacks[slot];

    encoder.copy_texture_to_buffer(
      self.texture.as_image_copy(),
      wgpu::TexelCopyBufferInfo {
        buffer: readback.get_buffer(),
        layout: wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
          rows_per_image: Some(1),
        },
      },
      wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
      },
    );
    readback.mark_recorded();

    Some(slot)
  }

  /// Asks for a readback's texel, its frame just submitted.
  pub fn request(&self, slot: usize) {
    self.readbacks[slot].request();
  }

  /// A readback's texel once it is back: kind, cluster, place and the depth's bits.
  pub fn take(&self, slot: usize) -> Option<XrfResult<[u32; 4]>> {
    self.readbacks[slot].take(|bytes| bytemuck::pod_read_unaligned::<[u32; 4]>(&bytes[..16]))
  }
}
