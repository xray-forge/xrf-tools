use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::enhanced_bloom_uniform::EnhancedBloomUniform;

/// What a stage of the enhanced bloom reads: the image it builds, blurs, halves, doubles or finishes, the coarser
/// halving a doubling adds, their sampler, the depth and marks the build finds the sky and self-lit surfaces by, and its
/// uniform.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct EnhancedBloomParameters<'a> {
  #[texture(d2, float)]
  pub source: GraphTexture,
  #[texture(d2, float)]
  pub coarser: GraphTexture,
  #[sampler(filtering)]
  pub source_sampler: &'a wgpu::Sampler,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[uniform]
  pub bloom: UniformBinding<EnhancedBloomUniform>,
}
