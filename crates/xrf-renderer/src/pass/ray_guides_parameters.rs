use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;

/// What DLSS ray reconstruction's guides are worked out from: the G-buffer, the wet surface, the lighting, and the
/// reflections' trace, whose alpha holds how far each traced ray went.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct RayGuidesParameters {
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub wet_surface: GraphTexture,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  /// What each traced pixel met, at the size the reflections are traced at; a texel of nothing where none is.
  #[texture(d2, unfilterable)]
  pub traced: GraphTexture,
}
