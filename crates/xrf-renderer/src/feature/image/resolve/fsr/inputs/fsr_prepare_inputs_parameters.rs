use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::feature::image::resolve::fsr::fsr_uniform::FsrUniform;

/// FSR's input preparation: the depth, motion and colour drawn.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub(crate) struct FsrPrepareInputsParameters {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub color: GraphTexture,
}
