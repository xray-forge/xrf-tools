/// What a graph texture is: its shape and format. Its usage is not stated; the graph gathers it from the accesses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GraphTextureDescriptor {
  pub label: &'static str,
  pub size: wgpu::Extent3d,
  pub mip_level_count: u32,
  pub sample_count: u32,
  pub dimension: wgpu::TextureDimension,
  pub format: wgpu::TextureFormat,
}

impl GraphTextureDescriptor {
  /// A two-dimensional texture of one mip and one layer.
  pub fn new_2d(label: &'static str, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
    Self {
      label,
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format,
    }
  }

  /// What `texture` is, as an import of it is declared.
  pub fn from_texture(label: &'static str, texture: &wgpu::Texture) -> Self {
    Self {
      label,
      size: texture.size(),
      mip_level_count: texture.mip_level_count(),
      sample_count: texture.sample_count(),
      dimension: texture.dimension(),
      format: texture.format(),
    }
  }

  pub fn with_mip_level_count(mut self, count: u32) -> Self {
    self.mip_level_count = count;
    self
  }

  pub fn with_array_layer_count(mut self, count: u32) -> Self {
    self.size.depth_or_array_layers = count;
    self
  }

  /// Bytes its texels take over every mip and layer, as near as the format says; for the graph's report.
  pub fn get_byte_size(&self) -> u64 {
    let (block_width, block_height) = self.format.block_dimensions();
    let block_bytes: u64 = u64::from(
      self
        .format
        .block_copy_size(None)
        .or_else(|| self.format.block_copy_size(Some(wgpu::TextureAspect::DepthOnly)))
        .unwrap_or(4),
    );
    let layers: u64 = u64::from(self.size.depth_or_array_layers);

    (0..self.mip_level_count)
      .map(|mip| {
        let width: u64 = u64::from((self.size.width >> mip).max(1).div_ceil(block_width));
        let height: u64 = u64::from((self.size.height >> mip).max(1).div_ceil(block_height));

        width * height * block_bytes * layers * u64::from(self.sample_count)
      })
      .sum()
  }
}
