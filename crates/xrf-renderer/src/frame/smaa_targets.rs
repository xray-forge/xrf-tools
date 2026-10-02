/// SMAA's edges and blending weights for a viewport's scene as drawn.
pub struct SmaaTargets {
  pub edges: wgpu::TextureView,
  pub weights: wgpu::TextureView,
}

impl SmaaTargets {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

  pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
    let create = |label: &str| -> wgpu::TextureView {
      device
        .create_texture(&wgpu::TextureDescriptor {
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
        .create_view(&Default::default())
    };

    Self {
      edges: create("smaa edges"),
      weights: create("smaa weights"),
    }
  }
}
