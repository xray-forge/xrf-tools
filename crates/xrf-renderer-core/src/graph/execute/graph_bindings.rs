use std::collections::HashMap;

use crate::graph::execute::graph_resolved_texture::GraphResolvedTexture;
use crate::graph::frame_graph::FrameGraph;
use crate::graph::resource::{GraphBuffer, GraphBufferDescriptor, GraphTexture, GraphTextureDescriptor};

/// The real resources behind a graph's imports, given when it executes.
#[derive(Default)]
pub struct GraphBindings<'r> {
  pub(crate) textures: HashMap<GraphTexture, GraphResolvedTexture<'r>>,
  pub(crate) buffers: HashMap<GraphBuffer, &'r wgpu::Buffer>,
}

impl<'r> GraphBindings<'r> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn bind_texture(&mut self, texture: GraphTexture, resolved: GraphResolvedTexture<'r>) -> &mut Self {
    self.textures.insert(texture, resolved);
    self
  }

  pub fn bind_buffer(&mut self, buffer: GraphBuffer, resolved: &'r wgpu::Buffer) -> &mut Self {
    self.buffers.insert(buffer, resolved);
    self
  }

  /// Imports the texture `view` views into `graph`, declared as it is, and binds it to that view.
  pub fn import_view(
    &mut self,
    graph: &mut FrameGraph<'_>,
    label: &'static str,
    view: &'r wgpu::TextureView,
  ) -> GraphTexture {
    let resolved: GraphResolvedTexture<'r> = GraphResolvedTexture::from_view(view);
    let texture: GraphTexture = graph.import_texture(GraphTextureDescriptor::from_texture(label, resolved.texture));

    self.bind_texture(texture, resolved);
    texture
  }

  /// Imports `buffer` into `graph`, declared at its size, and binds it.
  pub fn import_buffer(
    &mut self,
    graph: &mut FrameGraph<'_>,
    label: &'static str,
    buffer: &'r wgpu::Buffer,
  ) -> GraphBuffer {
    let imported: GraphBuffer = graph.import_buffer(GraphBufferDescriptor::new(label, buffer.size()));

    self.bind_buffer(imported, buffer);
    imported
  }
}
