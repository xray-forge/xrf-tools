use xrf_renderer_core::{GraphTexture, UniformBinding};

use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::scene::level::feature::water_reflection::WaterReflection;

/// What the enhanced water's reflection is traced and resolved from: the water's uniform, the frame copied before the
/// water, the empty texture standing in for what is not drawn, the reflection's targets, its two histories as the
/// frame's graph holds them (written, then read), and the blue noise its rays jitter by.
#[derive(Clone, Copy)]
pub struct WaterReflectionInputs<'a> {
  pub enhanced: UniformBinding<EnhancedWaterUniform>,
  pub scene: GraphTexture,
  pub nothing: GraphTexture,
  pub reflection: &'a WaterReflection,
  pub histories: [GraphTexture; 2],
  pub blue_noise: GraphTexture,
}
