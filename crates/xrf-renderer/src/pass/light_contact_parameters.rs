use xrf_renderer_core::{GraphTexture, PassParameters};

/// What the light contact pass writes at half the frame's size: the lights' contact shadows, a light's record and the
/// share of it kept a slot, four slots a layer; and the weight a light must reach there to be among those marched
/// towards, which the visibility and the lights find the marched lights by.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 3)]
pub struct LightContactParameters {
  #[storage_texture(d2_array, rgba32uint, write)]
  pub light_contact_out: GraphTexture,
  #[storage_texture(d2, r32float, write)]
  pub light_threshold_out: GraphTexture,
}
