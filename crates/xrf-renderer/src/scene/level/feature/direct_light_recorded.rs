use xrf_renderer_core::{GraphTexture, UniformBinding};

use crate::pass::contact_shadow_uniform::ContactShadowUniform;

/// What the sun's light answers a frame: how much of the sun reaches each pixel, and what the lights march their
/// contact shadows by.
#[derive(Clone, Copy)]
pub struct DirectLightRecorded {
  pub sun_seen: GraphTexture,
  pub contact: UniformBinding<ContactShadowUniform>,
  /// Lights each texel marches contact shadows towards at most: none while the lights march none.
  pub contact_lights: u32,
}
