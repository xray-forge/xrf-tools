/// The frame's depth reduced to the nearest depth under each texel at the size the reflections trace at: what their rays
/// are marched over.
pub struct ReflectionDepth {
  pub width: u32,
  pub height: u32,
  pub view: wgpu::TextureView,
}

impl ReflectionDepth {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

  pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("reflection depth"),
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Self::FORMAT,
      usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });

    Self {
      width,
      height,
      view: texture.create_view(&Default::default()),
    }
  }

  pub fn is_sized(&self, width: u32, height: u32) -> bool {
    self.width == width && self.height == height
  }
}
