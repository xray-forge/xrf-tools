use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// The shading change pyramid's first level: the dilated motion, and this frame's and the last frame's luma.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrShadingPyramidParameters {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub dilated_motion: GraphTexture,
  #[texture(d2, unfilterable)]
  pub luma: GraphTexture,
  #[texture(d2, unfilterable)]
  pub previous_luma: GraphTexture,
}
