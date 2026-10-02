/// The temporal resolve's two histories, colour and distance at the output's size: one read while the other is
/// written, swapped every frame.
pub struct TemporalHistory {
  pub textures: [wgpu::Texture; 2],
  pub views: [wgpu::TextureView; 2],
  /// The one this frame writes.
  pub index: usize,
  /// Whether the one this frame reads holds a frame.
  pub is_valid: bool,
}

impl TemporalHistory {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
    let textures: [wgpu::Texture; 2] = ["temporal history 0", "temporal history 1"].map(|label| {
      device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
          width,
          height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Self::FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
          | wgpu::TextureUsages::TEXTURE_BINDING
          | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
      })
    });
    let views: [wgpu::TextureView; 2] = [
      textures[0].create_view(&Default::default()),
      textures[1].create_view(&Default::default()),
    ];

    Self {
      textures,
      views,
      index: 0,
      is_valid: false,
    }
  }

  pub fn is_sized(&self, width: u32, height: u32) -> bool {
    let size: wgpu::Extent3d = self.textures[0].size();

    size.width == width && size.height == height
  }

  /// The written one becomes the one read.
  pub fn swap(&mut self) {
    self.index = 1 - self.index;
    self.is_valid = true;
  }
}
