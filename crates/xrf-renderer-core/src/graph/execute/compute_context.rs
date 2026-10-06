use crate::graph::execute::graph_pass_scope::GraphPassScope;
use crate::graph::execute::graph_resolved_texture::GraphResolvedTexture;
use crate::graph::resource::{GraphBuffer, GraphTexture};
use crate::param::PassParameters;

/// What a compute pass records with: the compute pass the graph opened and the resources the pass declared.
pub struct ComputeContext<'c> {
  pub(crate) pass: &'c mut wgpu::ComputePass<'static>,
  pub(crate) scope: GraphPassScope<'c>,
}

impl<'c> ComputeContext<'c> {
  pub fn get_pass(&mut self) -> &mut wgpu::ComputePass<'static> {
    self.pass
  }

  pub fn get_texture(&self, texture: GraphTexture) -> GraphResolvedTexture<'c> {
    self.scope.get_texture(texture)
  }

  pub fn get_buffer(&self, buffer: GraphBuffer) -> &'c wgpu::Buffer {
    self.scope.get_buffer(buffer)
  }

  /// Binds the bind group of `parameters` at the index they declare, with their dynamic offsets.
  pub fn bind<P: PassParameters>(&mut self, parameters: &P) {
    let (group, offsets) = self.scope.get_bind_group(parameters);

    self.pass.set_bind_group(P::GROUP, &group, &offsets);
  }
}
