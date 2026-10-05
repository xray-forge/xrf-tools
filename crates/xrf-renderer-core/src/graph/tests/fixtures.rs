use crate::graph::{
  FrameGraph, GraphBufferAccess, GraphBufferDescriptor, GraphColorAttachment, GraphTexture, GraphTextureAccess,
  GraphTextureDescriptor,
};

pub const COLOR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub fn color_texture(label: &'static str) -> GraphTextureDescriptor {
  GraphTextureDescriptor::new_2d(label, 64, 64, COLOR)
}

pub fn clear(texture: GraphTexture) -> GraphColorAttachment {
  GraphColorAttachment::new(texture, wgpu::LoadOp::Clear(wgpu::Color::BLACK))
}

pub fn load(texture: GraphTexture) -> GraphColorAttachment {
  GraphColorAttachment::new(texture, wgpu::LoadOp::Load)
}

/// A raster pass clearing `target` and sampling `sources`.
pub fn draw(graph: &mut FrameGraph<'_>, name: &'static str, target: GraphTexture, sources: &[GraphTexture]) {
  let mut builder = graph.add_raster_pass(name).color(clear(target));

  for source in sources {
    builder = builder.texture(*source, GraphTextureAccess::Sampled);
  }

  builder.record(|_| {});
}

/// An encoder pass copying `source` into an imported buffer, which keeps it and whatever it reads.
pub fn present(graph: &mut FrameGraph<'_>, source: GraphTexture) {
  let output = graph.import_buffer(GraphBufferDescriptor::new("output", 64 * 64 * 4));

  graph
    .add_encoder_pass("present")
    .texture(source, GraphTextureAccess::CopySource)
    .buffer(output, GraphBufferAccess::CopyDestination)
    .record(|_| {});
}
