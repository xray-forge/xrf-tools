/// What one water batch draws with: the nearest water's depth, then the surface as the engine draws it or enhanced.
pub struct WaterBatchPipelines {
  pub depth: wgpu::RenderPipeline,
  pub engine: wgpu::RenderPipeline,
  pub enhanced: wgpu::RenderPipeline,
}
