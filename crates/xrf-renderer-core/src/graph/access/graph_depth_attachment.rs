use crate::graph::resource::GraphTexture;

/// A raster pass's depth target: one mip of one layer, cleared, loaded, or only read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphDepthAttachment {
  pub texture: GraphTexture,
  pub load: wgpu::LoadOp<f32>,
  /// Tested against but never written, so the same depth may be sampled meanwhile.
  pub is_read_only: bool,
  pub mip_level: u32,
  pub array_layer: u32,
}

impl GraphDepthAttachment {
  pub fn new(texture: GraphTexture, load: wgpu::LoadOp<f32>) -> Self {
    Self {
      texture,
      load,
      is_read_only: false,
      mip_level: 0,
      array_layer: 0,
    }
  }

  /// Depth tested but not written; it keeps what it holds.
  pub fn new_read_only(texture: GraphTexture) -> Self {
    Self {
      is_read_only: true,
      ..Self::new(texture, wgpu::LoadOp::Load)
    }
  }

  pub fn with_mip_level(mut self, mip_level: u32) -> Self {
    self.mip_level = mip_level;
    self
  }

  pub fn with_array_layer(mut self, array_layer: u32) -> Self {
    self.array_layer = array_layer;
    self
  }

  /// Whether it targets the same texels the same way as another, whatever either does to them first.
  pub fn is_same_target(&self, other: &Self) -> bool {
    self.texture == other.texture
      && self.mip_level == other.mip_level
      && self.array_layer == other.array_layer
      && self.is_read_only == other.is_read_only
  }
}
