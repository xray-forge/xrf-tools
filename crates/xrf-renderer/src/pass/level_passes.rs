use crate::pass::ambient_occlusion_pass::AmbientOcclusionPass;
use crate::pass::combine_pass::CombinePass;
use crate::pass::depth_pyramid_pass::DepthPyramidPass;
use crate::pass::exposure_pass::ExposurePass;
use crate::pass::lights_pass::LightsPass;
use crate::pass::material_table::MaterialTable;
use crate::pass::present_pass::PresentPass;
use crate::pass::static_cull_pass::StaticCullPass;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_shadow_pass::StaticShadowPass;
use crate::pass::sun_pass::SunPass;

/// The passes a level's frame runs, borrowed from the GPU state together.
#[derive(Clone, Copy)]
pub struct LevelPasses<'a> {
  pub cull: &'a StaticCullPass,
  pub gbuffer: &'a StaticGBufferPass,
  pub shadow: &'a StaticShadowPass,
  pub pyramid: &'a DepthPyramidPass,
  pub sun: &'a SunPass,
  pub ambient_occlusion: &'a AmbientOcclusionPass,
  pub lights: &'a LightsPass,
  pub combine: &'a CombinePass,
  pub exposure: &'a ExposurePass,
  pub present: &'a PresentPass,
  pub table: &'a MaterialTable,
}
