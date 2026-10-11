use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR's debug views: the masks, the new locks, the luma instability, and this frame's history.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrDebugViewParameters {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, unfilterable)]
  pub masks: GraphTexture,
  #[texture(d2, unfilterable)]
  pub new_locks: GraphTexture,
  #[texture(d2, unfilterable)]
  pub luma_instability: GraphTexture,
  #[texture(d2, unfilterable)]
  pub history: GraphTexture,
}
