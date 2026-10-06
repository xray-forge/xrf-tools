/// A viewport's depth reduced level by level: to the farthest depth under each texel for occlusion tests, or to the
/// nearest for reflected rays to skip empty space by.
pub struct DepthPyramid {
  pub width: u32,
  pub height: u32,
  pub levels: u32,
  /// Every level, as the culls sample it.
  pub view: wgpu::TextureView,
  /// Each level alone, as its reduction reads and writes it.
  pub level_views: Vec<wgpu::TextureView>,
}

impl DepthPyramid {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

  /// The farthest depth's pyramid over a depth of this size: its first level the largest power of two within it each
  /// way, down to a texel.
  pub fn new(device: &wgpu::Device, depth_width: u32, depth_height: u32) -> Self {
    let width: u32 = previous_power_of_two(depth_width);
    let height: u32 = previous_power_of_two(depth_height);

    Self::create(device, "depth pyramid", (width, height), width.max(height).ilog2() + 1)
  }

  /// The nearest depth's pyramid over a depth of this size: its first level the depth itself, each next half the last
  /// rounded down, at most `levels` of them.
  pub fn new_nearest(device: &wgpu::Device, depth_width: u32, depth_height: u32, levels: u32) -> Self {
    let (width, height): (u32, u32) = (depth_width.max(1), depth_height.max(1));

    Self::create(
      device,
      "nearest depth pyramid",
      (width, height),
      levels.min(width.max(height).ilog2() + 1),
    )
  }

  fn create(device: &wgpu::Device, label: &str, (width, height): (u32, u32), levels: u32) -> Self {
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some(label),
      size: wgpu::Extent3d {
        width,
        height,
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
      width,
      height,
      levels,
      view: texture.create_view(&Default::default()),
      level_views,
    }
  }

  /// The size of one level.
  pub fn get_level_size(&self, level: u32) -> (u32, u32) {
    ((self.width >> level).max(1), (self.height >> level).max(1))
  }
}

fn previous_power_of_two(value: u32) -> u32 {
  if value <= 1 { 1 } else { 1 << value.ilog2() }
}
