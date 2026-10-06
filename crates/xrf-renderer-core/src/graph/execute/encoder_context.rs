use crate::graph::execute::graph_pass_scope::GraphPassScope;
use crate::graph::execute::graph_resolved_texture::GraphResolvedTexture;
use crate::graph::execute::pass_marker::PassMarker;
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// What an encoder pass records with: the encode group's encoder and the resources the pass declared.
pub struct EncoderContext<'c> {
  pub(crate) encoder: &'c mut wgpu::CommandEncoder,
  pub(crate) scope: GraphPassScope<'c>,
  pub(crate) marker: PassMarker<'c>,
}

impl<'c> EncoderContext<'c> {
  pub fn get_encoder(&mut self) -> &mut wgpu::CommandEncoder {
    self.encoder
  }

  /// The encoder and the marker timing the pass's own stages, together, as a bridge recording several passes needs.
  pub fn split(&mut self) -> (&mut wgpu::CommandEncoder, &mut PassMarker<'c>) {
    (self.encoder, &mut self.marker)
  }

  pub fn get_texture(&self, texture: GraphTexture) -> GraphResolvedTexture<'c> {
    self.scope.get_texture(texture)
  }

  pub fn get_buffer(&self, buffer: GraphBuffer) -> &'c wgpu::Buffer {
    self.scope.get_buffer(buffer)
  }
}
