/// A rain cover reduced level by level, from half its size down to a texel: each texel the lowest and the highest depth
/// of the cover under it, which the wetting tells by whether anything, or everything, stands over a point's
/// surroundings before it looks closer.
pub struct CoverPyramid {
  /// Texels its first level is across.
  pub size: u32,
  pub levels: u32,
  /// Every level, as the wetting reads it.
  pub view: wgpu::TextureView,
  /// Each level alone, as its reduction reads and writes it.
  pub level_views: Vec<wgpu::TextureView>,
}

impl CoverPyramid {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg32Float;

  /// The pyramid of a cover `resolution` texels across, a power of two.
  pub fn new(device: &wgpu::Device, label: &str, resolution: u32) -> Self {
    let size: u32 = (resolution / 2).max(1);
    let levels: u32 = size.ilog2() + 1;
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some(label),
      size: wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
      },
      mip_level_count: levels,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Self::FORMAT,
      usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });
    let level_views: Vec<wgpu::TextureView> = (0..levels)
      .map(|level| {
        texture.create_view(&wgpu::TextureViewDescriptor {
          base_mip_level: level,
          mip_level_count: Some(1),
          ..Default::default()
        })
      })
      .collect();

    Self {
      size,
      levels,
      view: texture.create_view(&Default::default()),
      level_views,
    }
  }

  /// Texels one level is across.
  pub fn get_level_size(&self, level: u32) -> u32 {
    (self.size >> level).max(1)
  }
}
