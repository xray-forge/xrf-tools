use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR 2's reactive mask: the frame before the water and the blended surfaces, and after.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrReactiveParameters {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub opaque: GraphTexture,
  #[texture(d2, unfilterable)]
  pub color: GraphTexture,
}
