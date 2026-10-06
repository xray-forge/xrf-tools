use crate::graph::{GraphBuffer, GraphResolvedTexture, GraphTexture};

/// The graph resources a pass's parameters resolve against while it records: those the pass declared.
pub trait PassResources<'r> {
  fn get_texture(&self, texture: GraphTexture) -> GraphResolvedTexture<'r>;

  fn get_buffer(&self, buffer: GraphBuffer) -> &'r wgpu::Buffer;
}
