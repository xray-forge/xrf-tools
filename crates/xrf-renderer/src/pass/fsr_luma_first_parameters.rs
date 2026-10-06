use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR 2's first luma step: the frame as drawn, filtered.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrLumaFirstParameters<'a> {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, float)]
  pub color: GraphTexture,
  #[sampler(filtering)]
  pub linear_sampler: &'a wgpu::Sampler,
}
