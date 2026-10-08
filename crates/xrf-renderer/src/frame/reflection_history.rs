/// The screen-space reflections' two blends over frames at the size they trace at, each the reflection and the distance
/// along the view of the point it stood at: one pair read while the other is written, swapped every frame.
pub struct ReflectionHistory {
  pub colours: [wgpu::Texture; 2],
  pub colour_views: [wgpu::TextureView; 2],
  pub helds: [wgpu::Texture; 2],
  pub held_views: [wgpu::TextureView; 2],
  /// The pair this frame writes.
  pub index: usize,
  /// Whether the pair this frame reads holds a frame.
  pub is_valid: bool,
}

impl ReflectionHistory {
  pub const COLOUR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const HELD_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

  pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
    let create = |label: &str, format: wgpu::TextureFormat| -> wgpu::Texture {
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
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
    };
    let colours: [wgpu::Texture; 2] =
      ["reflection history 0", "reflection history 1"].map(|label| create(label, Self::COLOUR_FORMAT));
    let helds: [wgpu::Texture; 2] =
      ["reflection history held 0", "reflection history held 1"].map(|label| create(label, Self::HELD_FORMAT));
    let colour_views: [wgpu::TextureView; 2] = [
      colours[0].create_view(&Default::default()),
      colours[1].create_view(&Default::default()),
    ];
    let held_views: [wgpu::TextureView; 2] = [
      helds[0].create_view(&Default::default()),
      helds[1].create_view(&Default::default()),
    ];

    Self {
      colours,
      colour_views,
      helds,
      held_views,
      index: 0,
      is_valid: false,
    }
  }

  pub fn is_sized(&self, width: u32, height: u32) -> bool {
    self.get_size() == (width, height)
  }

  /// The size it is traced at.
  pub fn get_size(&self) -> (u32, u32) {
    let size: wgpu::Extent3d = self.colours[0].size();

    (size.width, size.height)
  }

  /// The written pair becomes the one read.
  pub fn swap(&mut self) {
    self.index = 1 - self.index;
    self.is_valid = true;
  }
}
