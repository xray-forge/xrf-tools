use crate::graph::resource::GraphTextureDescriptor;

/// What the graph holds of one texture: its descriptor, whether it comes from outside the frame, and the usage its
/// accesses add up to.
#[derive(Clone, Debug)]
pub(crate) struct GraphTextureRecord {
  pub descriptor: GraphTextureDescriptor,
  pub is_imported: bool,
  pub usage: wgpu::TextureUsages,
}
