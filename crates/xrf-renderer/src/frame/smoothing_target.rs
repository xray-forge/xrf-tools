/// Where a smoothing pass writes a viewport's scene as drawn, to be copied back over it: it cannot read and write the
/// scene at once.
pub struct SmoothingTarget {
  pub texture: wgpu::Texture,
  pub view: wgpu::TextureView,
}

impl SmoothingTarget {
  pub fn new(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("smoothed"),
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });

    Self {
      view: texture.create_view(&Default::default()),
      texture,
    }
  }
}
