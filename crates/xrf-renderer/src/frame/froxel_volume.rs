/// The enhanced volumetric lighting's froxels, a grid along the view: what each scatters and how thick it is, kept over
/// frames in two (one read while the other is written, swapped every frame), and what is scattered towards the camera
/// up to each, integrated from them.
pub struct FroxelVolume {
  pub histories: [wgpu::Texture; 2],
  pub history_views: [wgpu::TextureView; 2],
  pub integrated: wgpu::Texture,
  pub integrated_view: wgpu::TextureView,
  /// The history this frame writes.
  pub index: usize,
  /// Whether the history this frame reads holds a frame.
  pub is_valid: bool,
  /// Frames written, which the froxels' jitter along the view turns by.
  pub frame: u32,
}

impl FroxelVolume {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// Froxels across the screen, down it and along the view.
  pub const SIZE: [u32; 3] = [160, 90, 64];

  pub fn new(device: &wgpu::Device) -> Self {
    let create = |label: &str| -> wgpu::Texture {
      device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
          width: Self::SIZE[0],
          height: Self::SIZE[1],
          depth_or_array_layers: Self::SIZE[2],
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: Self::FORMAT,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
    };
    let histories: [wgpu::Texture; 2] = [create("froxel history 0"), create("froxel history 1")];
    let integrated: wgpu::Texture = create("froxels integrated");

    Self {
      history_views: [
        histories[0].create_view(&Default::default()),
        histories[1].create_view(&Default::default()),
      ],
      integrated_view: integrated.create_view(&Default::default()),
      histories,
      integrated,
      index: 0,
      is_valid: false,
      frame: 0,
    }
  }

  /// The written history becomes the one read.
  pub fn swap(&mut self) {
    self.index = 1 - self.index;
    self.is_valid = true;
    self.frame = self.frame.wrapping_add(1);
  }
}
