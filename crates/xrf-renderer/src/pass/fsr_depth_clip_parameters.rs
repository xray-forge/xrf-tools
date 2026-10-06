use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR 2's depth clip: the frame, the motion drawn, the reconstructed previous depth, this frame's and the last one's
/// dilations, and the reactive mask.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrDepthClipParameters<'a> {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub color: GraphTexture,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
  #[storage]
  pub reconstructed: StorageArray<u32>,
  #[texture(d2, unfilterable)]
  pub dilated_depth: GraphTexture,
  #[texture(d2, unfilterable)]
  pub dilated_motion: GraphTexture,
  #[texture(d2, float)]
  pub previous_dilated_motion: GraphTexture,
  #[texture(d2, unfilterable)]
  pub reactive_mask: GraphTexture,
  #[sampler(filtering)]
  pub linear_sampler: &'a wgpu::Sampler,
}
