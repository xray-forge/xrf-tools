use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::contact_shadow_uniform::ContactShadowUniform;

/// What the contact shadows' march reads: the G-buffer's normals and depth, and its settings.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct ContactShadowParameters {
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[uniform]
  pub contact: UniformBinding<ContactShadowUniform>,
}
