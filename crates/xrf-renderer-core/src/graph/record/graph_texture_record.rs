use crate::graph::graph_owner::GraphOwner;
use crate::graph::resource::GraphTextureDescriptor;

/// What the graph holds of one texture: its descriptor, whether it comes from outside the frame, the usage its
/// accesses add up to, whose it was declared under, and what it was tagged with.
#[derive(Clone, Debug)]
pub(crate) struct GraphTextureRecord {
  pub descriptor: GraphTextureDescriptor,
  pub is_imported: bool,
  pub usage: wgpu::TextureUsages,
  /// The graph's owner when the texture was made or imported, as a pass takes it.
  pub owner: GraphOwner,
  /// What its declaration tagged it with (`FrameGraph::tag_texture`).
  pub tags: Vec<&'static str>,
}
