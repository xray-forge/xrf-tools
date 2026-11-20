use xrf_renderer_core::{GraphTexture, UniformBinding};

use crate::feature::weather::water::enhanced::enhanced_water_uniform::EnhancedWaterUniform;
use crate::feature::weather::water::reflection::water_reflection::WaterReflection;

/// What the enhanced water's reflection is traced and resolved from.
#[derive(Clone, Copy)]
pub(crate) struct WaterReflectionInputs<'a> {
  pub enhanced: UniformBinding<EnhancedWaterUniform>,
  /// The frame copied before the water.
  pub scene: GraphTexture,
  /// The empty texture standing in for what is not drawn.
  pub nothing: GraphTexture,
  pub reflection: &'a WaterReflection,
  /// The two histories as the frame's graph holds them, written then read.
  pub histories: [GraphTexture; 2],
}
