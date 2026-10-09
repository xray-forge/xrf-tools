use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;

/// What the grass's cull tests its cells' occlusion against: the static scene's depth pyramid of this frame, and the
/// view it was reduced through, its `has_history` none where the pyramid holds no depth of this frame.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct GrassOcclusionParameters {
  #[texture(d2, unfilterable)]
  pub pyramid: GraphTexture,
  #[uniform]
  pub occlusion: UniformBinding<StaticOcclusionUniform>,
}
