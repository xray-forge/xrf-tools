use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR's luma instability: the dilated motion, the masks, the luma, and the lumas the frame before kept.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrLumaInstabilityParameters<'a> {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub dilated_motion: GraphTexture,
  #[texture(d2, float)]
  pub masks: GraphTexture,
  #[texture(d2, float)]
  pub luma: GraphTexture,
  #[texture(d2, float)]
  pub previous_luma_history: GraphTexture,
  #[sampler(filtering)]
  pub linear_sampler: &'a wgpu::Sampler,
}
