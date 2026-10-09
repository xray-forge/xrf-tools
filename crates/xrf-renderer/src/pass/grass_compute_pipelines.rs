/// The grass's compute pipelines: planting the ring, then culling it, arranging the draws and sorting into them.
pub struct GrassComputePipelines {
  pub planting: Vec<wgpu::ComputePipeline>,
  /// Clearing the counts, then culling.
  pub culling: Vec<wgpu::ComputePipeline>,
  pub arrange: wgpu::ComputePipeline,
  pub sort: wgpu::ComputePipeline,
}
