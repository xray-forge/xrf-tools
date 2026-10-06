use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::water_blur_uniform::WaterBlurUniform;

/// What one way of the enhanced water's reflection blur reads: its source, the enhanced water's uniform, and its way.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct WaterBlurParameters<'a> {
  #[texture(d2, float)]
  pub source: GraphTexture,
  #[sampler(filtering)]
  pub source_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub enhanced: UniformBinding<EnhancedWaterUniform>,
  #[uniform]
  pub blur: UniformBinding<WaterBlurUniform>,
}
