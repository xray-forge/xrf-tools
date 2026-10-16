use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR's reactivity: the dilated depth and motion, the reconstructed previous depth, the reactive mask, the shading
/// change, the luma, the accumulation the frame before left, and the surface marks.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrPrepareReactivityParameters<'a> {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub dilated_depth: GraphTexture,
  #[texture(d2, unfilterable)]
  pub dilated_motion: GraphTexture,
  #[storage]
  pub reconstructed: StorageArray<u32>,
  #[texture(d2, unfilterable)]
  pub reactive_mask: GraphTexture,
  #[texture(d2, float)]
  pub shading_change: GraphTexture,
  #[texture(d2, unfilterable)]
  pub luma: GraphTexture,
  #[texture(d2, float)]
  pub previous_accumulation: GraphTexture,
  #[sampler(filtering)]
  pub linear_sampler: &'a wgpu::Sampler,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
}
