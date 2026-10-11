use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// One of FSR's downsamples: the level it halves.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrDownsampleParameters {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub source: GraphTexture,
}
