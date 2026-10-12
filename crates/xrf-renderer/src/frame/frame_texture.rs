/// A two-dimensional texture a feature owns across frames, with the view its passes bind.
pub struct FrameTexture {
  pub texture: wgpu::Texture,
  pub view: wgpu::TextureView,
}

impl FrameTexture {
  /// A texture of one level, `width` by `height` (each at least one), bound and drawn into, with `usage` besides.
  pub fn new(
    device: &wgpu::Device,
    label: &str,
    (width, height): (u32, u32),
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
  ) -> Self {
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some(label),
      size: wgpu::Extent3d {
        width: width.max(1),
        height: height.max(1),
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | usage,
      view_formats: &[],
    });

    Self {
      view: texture.create_view(&Default::default()),
      texture,
    }
  }

  pub fn get_size(&self) -> (u32, u32) {
    let size: wgpu::Extent3d = self.texture.size();

    (size.width, size.height)
  }

  /// Bytes its texels take.
  pub fn get_memory(&self) -> u64 {
    let size: wgpu::Extent3d = self.texture.size();
    let texel: u64 = u64::from(self.texture.format().block_copy_size(None).unwrap_or(4));

    u64::from(size.width) * u64::from(size.height) * u64::from(size.depth_or_array_layers) * texel
  }
}
