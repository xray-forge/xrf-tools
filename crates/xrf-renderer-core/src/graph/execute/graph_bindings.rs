use std::collections::HashMap;

use crate::graph::execute::graph_resolved_texture::GraphResolvedTexture;
use crate::graph::resource::{GraphBuffer, GraphTexture};

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
}
