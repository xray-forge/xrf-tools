use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR's shading change: the shading change pyramid's three levels.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrShadingChangeParameters<'a> {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, float)]
  pub level0: GraphTexture,
  #[texture(d2, float)]
  pub level1: GraphTexture,
  #[texture(d2, float)]
  pub level2: GraphTexture,
  #[sampler(filtering)]
  pub linear_sampler: &'a wgpu::Sampler,
}
