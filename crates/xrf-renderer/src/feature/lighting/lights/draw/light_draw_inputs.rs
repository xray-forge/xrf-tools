use xrf_renderer_core::UniformBinding;

use crate::feature::lighting::contact_shadows::contact_shadow_uniform::ContactShadowUniform;
use crate::feature::lighting::lights::draw::light_buffers::LightBuffers;
use crate::feature::tracing::view_trace::traced_scene::TracedScene;
use crate::feature::weather::wet::wet_targets::WetTargets;
use crate::scene::level::light_shadow_atlas::LightShadowAtlas;

/// What the lights' draw reads beside the view's record: the scene's shadow atlas, the binning's buffers, what the
/// wetting wrote, the uniform the contact shadows march by and how many of each texel's strongest lights they march
/// towards, and the scene's trace where the shadowed lights are traced, its rays turned by `noise_frame`.
#[derive(Clone, Copy)]
pub struct LightDrawInputs<'a> {
  pub atlas: LightShadowAtlas<'a>,
  pub buffers: LightBuffers,
  pub wet: WetTargets,
  pub contact: UniformBinding<ContactShadowUniform>,
  pub contact_lights: u32,
  pub traced: Option<TracedScene<'a>>,
  pub noise_frame: u32,
}
