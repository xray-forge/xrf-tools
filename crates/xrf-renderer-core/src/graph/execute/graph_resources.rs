use crate::graph::execute::graph_resolved_texture::GraphResolvedTexture;
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// Every resource of a compiled graph resolved for the frame: transients from the pool, imports from their bindings,
/// and the upload ring's buffer. A culled resource is resolved to nothing.
pub(crate) struct GraphResources<'r> {
  pub(crate) textures: Vec<Option<GraphResolvedTexture<'r>>>,
  pub(crate) buffers: Vec<Option<&'r wgpu::Buffer>>,
  pub(crate) upload: &'r wgpu::Buffer,
}

impl<'r> GraphResources<'r> {
  /// The texture behind a handle.
  ///
  /// # Panics
  ///
  /// When the handle names a texture no surviving pass declares, which a pass recording through its context cannot do.
  pub fn get_texture(&self, texture: GraphTexture) -> GraphResolvedTexture<'r> {
    self.textures[texture.get_index()].unwrap_or_else(|| panic!("Graph texture {} is not resolved", texture.index))
  }

  /// The buffer behind a handle.
  ///
  /// # Panics
  ///
  /// When the handle names a buffer no surviving pass declares, which a pass recording through its context cannot do.
  pub fn get_buffer(&self, buffer: GraphBuffer) -> &'r wgpu::Buffer {
    self.buffers[buffer.get_index()].unwrap_or_else(|| panic!("Graph buffer {} is not resolved", buffer.index))
  }
}
