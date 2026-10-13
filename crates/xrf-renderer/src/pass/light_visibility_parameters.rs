use xrf_renderer_core::{GraphTexture, PassParameters};

/// What the lights pass reads of the light visibility pass: how much of each of its shadowed lights a pixel sees, a
/// share a slot, four slots a layer.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 3)]
pub struct LightVisibilityParameters {
  #[texture(d2_array, unfilterable)]
  pub light_visibility: GraphTexture,
  /// The weight a light must reach to be among those the contact shadows marched towards, at half the frame's size.
  #[texture(d2, unfilterable)]
  pub light_threshold: GraphTexture,
}
