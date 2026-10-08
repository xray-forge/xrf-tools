use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::overhead_shift_uniform::OverheadShiftUniform;

/// What shifting an overhead map reads: its copy as it stood, and how far it moved.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct OverheadShiftParameters {
  #[texture(d2, depth)]
  pub previous: GraphTexture,
  #[uniform]
  pub shift: UniformBinding<OverheadShiftUniform>,
}
