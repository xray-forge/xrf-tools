use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;

/// What the overlays read of a viewport's frame: its depth, which hides them behind what is drawn, where it stands in
/// the window, and its lighting, which places the sun.
#[derive(PassParameters)]
#[parameters(group = 1)]
pub struct OverlayParameters {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[uniform]
  pub present: UniformBinding<PresentUniform>,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
}
