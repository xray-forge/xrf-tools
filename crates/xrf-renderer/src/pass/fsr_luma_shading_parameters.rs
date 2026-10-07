use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR 2's shading luma: the first step's luma.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrLumaShadingParameters {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub first_step: GraphTexture,
}
