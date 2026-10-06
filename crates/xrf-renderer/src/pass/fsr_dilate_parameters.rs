use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR 2's dilation: the depth and motion drawn, and the frame.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrDilateParameters {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub color: GraphTexture,
}
