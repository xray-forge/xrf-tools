use crate::pass::combine_pass::CombinePass;
use crate::pass::depth_pyramid_pass::DepthPyramidPass;
use crate::pass::static_cull_pass::StaticCullPass;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;

/// The passes a level's frame runs, borrowed from the GPU state together.
#[derive(Clone, Copy)]
pub struct LevelPasses<'a> {
  pub cull: &'a StaticCullPass,
  pub gbuffer: &'a StaticGBufferPass,
  pub pyramid: &'a DepthPyramidPass,
  pub combine: &'a CombinePass,
}
