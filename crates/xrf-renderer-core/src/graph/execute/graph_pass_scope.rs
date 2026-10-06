use crate::graph::execute::graph_resolved_texture::GraphResolvedTexture;
use crate::graph::execute::graph_resources::GraphResources;
use crate::graph::resource::{GraphBuffer, GraphTexture};
use crate::param::{BindGroupCache, PassParameters, PassResources};

/// The resources one pass may reach: the frame's, narrowed in debug builds to those the pass declared.
pub(crate) struct GraphPassScope<'c> {
  pub name: &'static str,
  pub device: &'c wgpu::Device,
  pub cache: &'c BindGroupCache,
  pub resources: &'c GraphResources<'c>,
  pub textures: &'c [GraphTexture],
  pub buffers: &'c [GraphBuffer],
}

impl<'c> GraphPassScope<'c> {
  pub fn get_texture(&self, texture: GraphTexture) -> GraphResolvedTexture<'c> {
    debug_assert!(
      self.textures.contains(&texture),
      "Pass '{}' reaches graph texture {} it did not declare",
      self.name,
      texture.index
    );

    self.resources.get_texture(texture)
  }

  pub fn get_buffer(&self, buffer: GraphBuffer) -> &'c wgpu::Buffer {
    debug_assert!(
      self.buffers.contains(&buffer),
      "Pass '{}' reaches graph buffer {} it did not declare",
      self.name,
      buffer.index
    );

    self.resources.get_buffer(buffer)
  }

  /// The bind group of `parameters`, from the cache, and its dynamic offsets.
  pub fn get_bind_group<P: PassParameters>(&self, parameters: &P) -> (wgpu::BindGroup, Vec<u32>) {
    (
      self.cache.get_bind_group(self.device, parameters, self),
      parameters.list_dynamic_offsets(),
    )
  }
}

impl<'c> PassResources<'c> for GraphPassScope<'c> {
  fn get_texture(&self, texture: GraphTexture) -> GraphResolvedTexture<'c> {
    GraphPassScope::get_texture(self, texture)
  }

  fn get_buffer(&self, buffer: GraphBuffer) -> &'c wgpu::Buffer {
    GraphPassScope::get_buffer(self, buffer)
  }
}
