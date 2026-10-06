use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::wet_uniform::WetUniform;

/// What laying the wet look over the normals and the albedo reads: the depth, the patches, and the settings.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct WetApplyParameters {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub patched: GraphTexture,
  #[uniform]
  pub wet: UniformBinding<WetUniform>,
}
