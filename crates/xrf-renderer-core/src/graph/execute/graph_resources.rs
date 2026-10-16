use std::collections::HashMap;

use crate::graph::execute::graph_resolved_texture::GraphResolvedTexture;
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// Every resource of a compiled graph resolved for the frame: transients from the pool, imports from their bindings,
/// the views of one mip and layer its render passes draw into, and the upload ring's buffer. A culled resource is
/// resolved to nothing.
pub(crate) struct GraphResources<'r> {
  pub(crate) textures: Vec<Option<GraphResolvedTexture<'r>>>,
  /// Each attachment of a texture with mips or layers, by the texture, mip and layer it draws into.
  pub(crate) attachments: HashMap<(GraphTexture, u32, u32), wgpu::TextureView>,
  pub(crate) buffers: Vec<Option<&'r wgpu::Buffer>>,
  pub(crate) upload: &'r wgpu::Buffer,
}

impl<'r> GraphResources<'r> {
  /// The texture behind a handle.
  ///
  /// # Panics
  ///
  /// When the handle names a texture no surviving pass declares. A pass's scope refuses an undeclared handle in debug
  /// builds; a release build reaches this only for one no pass declares at all.
  pub fn get_texture(&self, texture: GraphTexture) -> GraphResolvedTexture<'r> {
    self.textures[texture.get_index()].unwrap_or_else(|| panic!("Graph texture {} is not resolved", texture.index))
  }

  /// The view a render pass draws into `texture`'s `mip_level` and `array_layer` through: the texture's whole view
  /// where it has one mip and layer.
  pub fn get_attachment(&self, texture: GraphTexture, mip_level: u32, array_layer: u32) -> &wgpu::TextureView {
    self
      .attachments
      .get(&(texture, mip_level, array_layer))
      .unwrap_or_else(|| self.get_texture(texture).view)
  }

  /// The buffer behind a handle.
  ///
  /// # Panics
  ///
  /// When the handle names a buffer no surviving pass declares. A pass's scope refuses an undeclared handle in debug
  /// builds; a release build reaches this only for one no pass declares at all.
  pub fn get_buffer(&self, buffer: GraphBuffer) -> &'r wgpu::Buffer {
    self.buffers[buffer.get_index()].unwrap_or_else(|| panic!("Graph buffer {} is not resolved", buffer.index))
  }
}
