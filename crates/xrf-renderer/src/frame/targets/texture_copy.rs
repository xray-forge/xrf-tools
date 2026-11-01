use xrf_renderer_core::{FrameGraph, GraphTexture, GraphTextureAccess};

use crate::frame::targets::texture_layer::TextureLayer;

/// Adds a pass copying the whole of one texture into another of its size.
pub fn add_texture_copy(
  graph: &mut FrameGraph<'_>,
  name: &'static str,
  source: GraphTexture,
  destination: GraphTexture,
) {
  graph
    .add_encoder_pass(name)
    .texture(source, GraphTextureAccess::CopySource)
    .texture(destination, GraphTextureAccess::CopyDestination)
    .record(move |context| {
      let (source, destination) = (context.get_texture(source), context.get_texture(destination));

      context.get_encoder().copy_texture_to_texture(
        source.texture.as_image_copy(),
        destination.texture.as_image_copy(),
        source.texture.size(),
      );
    });
}

/// Adds a pass copying one layer of a layered texture, whole, into a layer of another whose layers are its size.
pub fn add_layer_copy(graph: &mut FrameGraph<'_>, name: &'static str, source: TextureLayer, destination: TextureLayer) {
  graph
    .add_encoder_pass(name)
    .texture(source.texture, GraphTextureAccess::CopySource)
    .texture(destination.texture, GraphTextureAccess::CopyDestination)
    .record(move |context| {
      let (from, into) = (
        context.get_texture(source.texture),
        context.get_texture(destination.texture),
      );
      let size: wgpu::Extent3d = wgpu::Extent3d {
        depth_or_array_layers: 1,
        ..into.texture.size()
      };

      context.get_encoder().copy_texture_to_texture(
        TextureLayer::to_copy(from.texture, source.layer),
        TextureLayer::to_copy(into.texture, destination.layer),
        size,
      );
    });
}
