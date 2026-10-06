/// VBAO's two accumulations at the search's half size, visibility, distance along the view
/// and frames gathered: one read while the other is written, swapped every frame.
pub struct VbaoHistory {
  pub textures: [wgpu::Texture; 2],
  pub views: [wgpu::TextureView; 2],
  /// The one this frame writes.
  pub index: usize,
  /// Whether the one this frame reads holds a frame.
  pub is_valid: bool,
  /// Frames written, which the search's noise turns by.
  pub frame: u32,
}

impl VbaoHistory {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
    let textures: [wgpu::Texture; 2] = ["vbao history 0", "vbao history 1"].map(|label| {
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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
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
      frame: 0,
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
    self.frame = self.frame.wrapping_add(1);
  }
}
