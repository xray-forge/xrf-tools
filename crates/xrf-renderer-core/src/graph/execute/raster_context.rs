use crate::graph::execute::graph_pass_scope::GraphPassScope;
use crate::graph::execute::graph_resolved_texture::GraphResolvedTexture;
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// What a raster pass records with: the render pass the graph opened, perhaps shared with the passes beside it, and the
/// resources the pass declared.
pub struct RasterContext<'c> {
  pub(crate) pass: &'c mut wgpu::RenderPass<'static>,
  pub(crate) scope: GraphPassScope<'c>,
}

impl<'c> RasterContext<'c> {
  pub fn get_pass(&mut self) -> &mut wgpu::RenderPass<'static> {
    self.pass
  }

  pub fn get_texture(&self, texture: GraphTexture) -> GraphResolvedTexture<'c> {
    self.scope.get_texture(texture)
  }

  pub fn get_buffer(&self, buffer: GraphBuffer) -> &'c wgpu::Buffer {
    self.scope.get_buffer(buffer)
  }
}
