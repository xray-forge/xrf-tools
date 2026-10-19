/// The rings the enhanced drops leave where they land, seen from overhead over the tile their columns repeat over,
/// wrapping: each texel the tilt the rings give the water in renderer `x` and `z` and that tilt's square, reduced level
/// by level so water seen far off reads them filtered.
pub struct RainRings {
  /// Texels its first level is across.
  pub size: u32,
  /// Every level, as the water and the puddles read it.
  pub view: wgpu::TextureView,
  /// Each level alone, as the rings are drawn into the first and it is reduced into the rest.
  pub level_views: Vec<wgpu::TextureView>,
}

impl RainRings {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  /// Metres a texel of the first level spans at most, two centimetres, so a ring's waves span a few of them; and the
  /// most texels it is across.
  const TEXEL: f32 = 0.02;
  const MOST: u32 = 1536;

  /// Texels a reduction's workgroup takes a side, which the first level is a multiple of (`frame/rain_rings.wgsl`).
  pub const BLOCK: u32 = 16;

  /// Levels, the first and the four one dispatch reduces it into (`frame/rain_rings.wgsl`).
  pub const LEVELS: u32 = 5;

  /// Texels the first level is across for a tile `tile` metres a side.
  pub fn get_size(tile: f32) -> u32 {
    let blocks: u32 = (tile / Self::TEXEL / Self::BLOCK as f32).ceil() as u32;

    (blocks * Self::BLOCK).clamp(Self::BLOCK << Self::LEVELS, Self::MOST)
  }

  /// The rings of a tile `size` texels across (`get_size`).
  pub fn new(device: &wgpu::Device, size: u32) -> Self {
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("rain rings"),
      size: wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
      },
      mip_level_count: Self::LEVELS,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Self::FORMAT,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::STORAGE_BINDING
        | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });
    let level_views: Vec<wgpu::TextureView> = (0..Self::LEVELS)
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
      view: texture.create_view(&Default::default()),
      level_views,
    }
  }

  /// Bytes its levels hold.
  pub fn get_memory(&self) -> u64 {
    (0..Self::LEVELS)
      .map(|level| u64::from((self.size >> level).pow(2)) * 8)
      .sum()
  }
}
