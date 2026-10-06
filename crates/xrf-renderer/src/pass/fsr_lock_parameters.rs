use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR 2's locks: this frame's lock luma.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrLockParameters {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub lock_luma: GraphTexture,
}
