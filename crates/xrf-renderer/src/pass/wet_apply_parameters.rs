use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::wet_uniform::WetUniform;

/// What laying the wet look over the normals and the albedo reads: the depth, the material marks, the patches, the
/// enhanced wetting's wet surface (a texel of nothing for the engine's), and the settings.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct WetApplyParameters {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub patched: GraphTexture,
  #[texture(d2, unfilterable)]
  pub wet_surface: GraphTexture,
  #[uniform]
  pub wet: UniformBinding<WetUniform>,
}
