use xrf_renderer_core::{FrameGraph, GraphTexture, GraphTextureAccess};

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
