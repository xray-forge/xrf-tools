use xrf_renderer_core::{GraphTexture, UniformBinding};

use crate::feature::weather::water::enhanced::enhanced_water_uniform::EnhancedWaterUniform;
use crate::feature::weather::water::reflection::water_reflection::WaterReflection;

/// What the enhanced water's reflection is traced and resolved from: the water's uniform, the frame copied before the
/// water, the empty texture standing in for what is not drawn, the reflection's targets, and its two histories as the
/// frame's graph holds them (written, then read).
#[derive(Clone, Copy)]
pub(crate) struct WaterReflectionInputs<'a> {
  pub enhanced: UniformBinding<EnhancedWaterUniform>,
  pub scene: GraphTexture,
  pub nothing: GraphTexture,
  pub reflection: &'a WaterReflection,
  pub histories: [GraphTexture; 2],
}
