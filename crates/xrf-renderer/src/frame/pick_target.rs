use xrf_error::{XrfError, XrfResult};

/// A pick's one texel and its depth, and the buffer it is read back through.
pub struct PickTarget {
  pub color: wgpu::TextureView,
  pub depth: wgpu::TextureView,
  texture: wgpu::Texture,
  readback: wgpu::Buffer,
}

impl PickTarget {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Uint;

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
      readback: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pick readback"),
        size: wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
      }),
      texture,
    }
  }

  /// Copies the texel out with the frame's work.
  pub fn copy_out(&self, encoder: &mut wgpu::CommandEncoder) {
    encoder.copy_texture_to_buffer(
      self.texture.as_image_copy(),
      wgpu::TexelCopyBufferInfo {
        buffer: &self.readback,
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
  }

  /// Waits for the frame's work and reads the texel: kind, cluster, place and the depth's bits.
  pub fn read(&self, device: &wgpu::Device) -> XrfResult<[u32; 4]> {
    self.readback.slice(..).map_async(wgpu::MapMode::Read, |_| ());
    device
      .poll(wgpu::PollType::wait_indefinitely())
      .map_err(|error| XrfError::new_unexpected_error(format!("Pick readback: {error}")))?;

    let texel: [u32; 4] = {
      let view = self
        .readback
        .slice(..)
        .get_mapped_range()
        .map_err(|error| XrfError::new_unexpected_error(format!("Pick readback: {error}")))?;

      bytemuck::pod_read_unaligned::<[u32; 4]>(&view[..16])
    };

    self.readback.unmap();

    Ok(texel)
  }
}
