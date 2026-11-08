use xrf_renderer_core::{GraphTexture, UniformBinding};

use crate::feature::lighting::contact_shadows::contact_shadow_uniform::ContactShadowUniform;

/// What the contact shadows answer a frame: the sun's, which the sun multiplies its own by, a lit texel where none are
/// marched, and the uniform the lights march theirs by.
#[derive(Clone, Copy)]
pub(crate) struct ContactShadowsRecorded {
  pub sun: GraphTexture,
  pub uniform: UniformBinding<ContactShadowUniform>,
}
