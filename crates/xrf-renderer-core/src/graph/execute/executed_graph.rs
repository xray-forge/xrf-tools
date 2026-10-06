use std::time::Duration;

use crate::graph::execute::executed_group::ExecutedGroup;

/// A frame graph recorded: a command buffer per encode group, in order for one submit, and what each group cost the
/// CPU.
pub struct ExecutedGraph {
  pub commands: Vec<wgpu::CommandBuffer>,
  pub groups: Vec<ExecutedGroup>,
  /// What wgpu's encoding cost the thread that executed the graph: the groups' `finish` summed where they recorded one
  /// after another, and the whole span of their recording where they recorded in parallel, which overlaps it.
  pub encode: Duration,
}
