use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::vbao_uniform::VbaoUniform;

/// What a stage of VBAO reads: the G-buffer's normals, depth and motion, its settings, the
/// stage before's result, and the last frame's accumulation.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct VbaoParameters {
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
  #[uniform]
  pub occlusion: UniformBinding<VbaoUniform>,
  #[texture(d2, unfilterable)]
  pub source: GraphTexture,
  #[texture(d2, unfilterable)]
  pub history: GraphTexture,
}
