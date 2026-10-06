use crate::graph::execute::executed_group::ExecutedGroup;

/// A frame graph recorded: a command buffer per encode group, in order for one submit, and what each group cost the
/// CPU.
pub struct ExecutedGraph {
  pub commands: Vec<wgpu::CommandBuffer>,
  pub groups: Vec<ExecutedGroup>,
}
