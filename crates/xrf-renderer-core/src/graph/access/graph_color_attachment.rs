use crate::graph::resource::GraphTexture;

/// A raster pass's colour target: one mip of one layer of a texture, cleared or loaded, always stored. A texture of
/// several mips or layers is drawn into only at those it names (`with_mip_level`, `with_array_layer`): the graph knows
/// the texture, never which part of it an imported view shows, so an unnamed target there is refused when the graph
/// resolves, rather than drawn into its first mip and layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphColorAttachment {
  pub texture: GraphTexture,
  pub load: wgpu::LoadOp<wgpu::Color>,
  pub mip_level: u32,
  pub array_layer: u32,
  /// Whether its mip and its layer were named.
  pub is_mip_named: bool,
  pub is_layer_named: bool,
}

impl GraphColorAttachment {
  pub fn new(texture: GraphTexture, load: wgpu::LoadOp<wgpu::Color>) -> Self {
    Self {
      texture,
      load,
      mip_level: 0,
      array_layer: 0,
      is_mip_named: false,
      is_layer_named: false,
    }
  }

  pub fn with_mip_level(mut self, mip_level: u32) -> Self {
    self.mip_level = mip_level;
    self.is_mip_named = true;
    self
  }

  pub fn with_array_layer(mut self, array_layer: u32) -> Self {
    self.array_layer = array_layer;
    self.is_layer_named = true;
    self
  }

  /// Whether it targets the same texels as another, whatever either does to them first.
  pub fn is_same_target(&self, other: &Self) -> bool {
    self.texture == other.texture && self.mip_level == other.mip_level && self.array_layer == other.array_layer
  }
}
