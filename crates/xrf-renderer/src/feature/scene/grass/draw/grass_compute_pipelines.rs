/// The grass's compute pipelines: planting the ring, then culling it, arranging the draws and sorting into them; and
/// standing the trace's cards.
pub struct GrassComputePipelines {
  pub planting: Vec<wgpu::ComputePipeline>,
  /// Clearing the counts, then culling.
  pub culling: Vec<wgpu::ComputePipeline>,
  pub arrange: wgpu::ComputePipeline,
  pub sort: wgpu::ComputePipeline,
  /// Clearing the cards, then standing them.
  pub cards: Vec<wgpu::ComputePipeline>,
}
