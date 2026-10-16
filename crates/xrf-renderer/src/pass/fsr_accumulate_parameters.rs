use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR's accumulation: the frame drawn, the dilated motion, the masks, the luma instability, the farthest depth at half
/// size, the new locks, and the last frame's history.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrAccumulateParameters<'a> {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub color: GraphTexture,
  #[texture(d2, unfilterable)]
  pub dilated_motion: GraphTexture,
  #[texture(d2, float)]
  pub masks: GraphTexture,
  #[texture(d2, float)]
  pub luma_instability: GraphTexture,
  #[texture(d2, float)]
  pub farthest_depth: GraphTexture,
  #[texture(d2, unfilterable)]
  pub new_locks: GraphTexture,
  #[texture(d2, unfilterable)]
  pub history: GraphTexture,
  #[sampler(filtering)]
  pub linear_sampler: &'a wgpu::Sampler,
}
