/// A viewport's frame at its own size where the scene is drawn smaller: upscaled into the first, sharpened into the
/// second.
pub struct UpscaleTargets {
  pub width: u32,
  pub height: u32,
  pub textures: [wgpu::Texture; 2],
  pub views: [wgpu::TextureView; 2],
}

impl UpscaleTargets {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
    let textures: [wgpu::Texture; 2] = ["upscaled", "sharpened"].map(|label| {
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
          | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
      })
    });
    let views: [wgpu::TextureView; 2] = [
      textures[0].create_view(&Default::default()),
      textures[1].create_view(&Default::default()),
    ];

    Self {
      width,
      height,
      textures,
      views,
    }
  }

  pub fn is_sized(&self, width: u32, height: u32) -> bool {
    self.width == width && self.height == height
  }
}
