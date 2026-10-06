use crate::graph::resource::GraphBufferDescriptor;

/// What the graph holds of one buffer: its descriptor, whether it comes from outside the frame, and the usage its
/// accesses add up to.
#[derive(Clone, Debug)]
pub(crate) struct GraphBufferRecord {
  pub descriptor: GraphBufferDescriptor,
  pub is_imported: bool,
  pub usage: wgpu::BufferUsages,
}
