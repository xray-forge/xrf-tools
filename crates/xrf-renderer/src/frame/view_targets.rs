/// One viewport's G-buffer, sized to its rectangle and made again when that changes.
pub struct ViewTargets {
  pub width: u32,
  pub height: u32,
  /// Albedo, then gloss.
  pub albedo: wgpu::TextureView,
  /// The view space normal, octahedral.
  pub normal: wgpu::TextureView,
  /// Hemisphere, sun, material slice.
  pub material: wgpu::TextureView,
  /// Reversed: one at the near plane, zero where nothing was drawn.
  pub depth: wgpu::TextureView,
}

impl ViewTargets {
  pub const ALBEDO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  pub const NORMAL: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;
  pub const MATERIAL: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

  pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
    let create = |label: &str, format: wgpu::TextureFormat| -> wgpu::TextureView {
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
          format,
          usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default())
    };

    Self {
      width,
      height,
      albedo: create("albedo", Self::ALBEDO),
      normal: create("normal", Self::NORMAL),
      material: create("material", Self::MATERIAL),
      depth: create("depth", Self::DEPTH),
    }
  }

  pub fn is_sized(&self, width: u32, height: u32) -> bool {
    self.width == width && self.height == height
  }
}
