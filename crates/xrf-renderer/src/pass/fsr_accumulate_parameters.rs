use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR 2's accumulation: the prepared frame and its masks, the motion, the locks, the shading luma, and the last
/// frame's history, lock status and luma history.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrAccumulateParameters<'a> {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, float)]
  pub prepared: GraphTexture,
  #[texture(d2, float)]
  pub reactive_masks: GraphTexture,
  #[texture(d2, unfilterable)]
  pub dilated_motion: GraphTexture,
  #[texture(d2, unfilterable)]
  pub locks: GraphTexture,
  #[texture(d2, float)]
  pub shading_luma: GraphTexture,
  #[texture(d2, unfilterable)]
  pub history: GraphTexture,
  #[texture(d2, float)]
  pub lock_status: GraphTexture,
  #[texture(d2, float)]
  pub luma_history: GraphTexture,
  #[sampler(filtering)]
  pub linear_sampler: &'a wgpu::Sampler,
}
