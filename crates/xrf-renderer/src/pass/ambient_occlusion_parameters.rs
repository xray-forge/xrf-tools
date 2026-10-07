use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;

/// What a stage of the ambient occlusion reads: the normals and depth, its settings, and the other occlusion target.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct AmbientOcclusionParameters {
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[uniform]
  pub occlusion: UniformBinding<AmbientOcclusionUniform>,
  #[texture(d2, unfilterable)]
  pub source: GraphTexture,
}
