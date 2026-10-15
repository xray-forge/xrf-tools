/// The composited pass's pipelines, each list one a layout or a batch.
pub struct CompositedPipelines {
  /// One a composited batch, in `StaticBatch::list_composited` order.
  pub composited: Vec<wgpu::RenderPipeline>,
  /// One a wall mark batch, in `StaticBatch::list_wallmarks` order.
  pub wallmarks: Vec<wgpu::RenderPipeline>,
  /// One a layout, in `StaticLayout::ALL` order, for the distortion.
  pub distortion: Vec<wgpu::RenderPipeline>,
  /// One a batch of either composited class, in `CompositedPass::list_selectable` order, for the selection's mark.
  pub selection: Vec<wgpu::RenderPipeline>,
}
