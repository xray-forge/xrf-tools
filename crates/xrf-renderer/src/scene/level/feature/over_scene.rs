use xrf_renderer_core::{FrameGraph, GraphColorAttachment, GraphDepthAttachment, RasterPassBuilder};

use crate::frame::view_target_handles::ViewTargetHandles;

/// Adds a raster pass blending over the scene, tested against its depth without writing it.
pub fn add_over_scene<'g, 'a>(
  graph: &'g mut FrameGraph<'a>,
  targets: ViewTargetHandles,
  name: &'static str,
) -> RasterPassBuilder<'g, 'a> {
  graph
    .add_raster_pass(name)
    .color(GraphColorAttachment::new(targets.scene, wgpu::LoadOp::Load))
    .depth(GraphDepthAttachment::new_read_only(targets.depth))
}
