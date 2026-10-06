use crate::graph::resource::GraphTextureDescriptor;

/// What makes two transient textures interchangeable: everything but the label, and the usage the graph gathered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransientTextureKey {
  pub size: wgpu::Extent3d,
  pub mip_level_count: u32,
  pub sample_count: u32,
  pub dimension: wgpu::TextureDimension,
  pub format: wgpu::TextureFormat,
  pub usage: wgpu::TextureUsages,
}

impl TransientTextureKey {
  pub fn new(descriptor: &GraphTextureDescriptor, usage: wgpu::TextureUsages) -> Self {
    Self {
      size: descriptor.size,
      mip_level_count: descriptor.mip_level_count,
      sample_count: descriptor.sample_count,
      dimension: descriptor.dimension,
      format: descriptor.format,
      usage,
    }
  }
}
