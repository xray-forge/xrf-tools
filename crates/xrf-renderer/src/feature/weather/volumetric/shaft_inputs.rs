use xrf_renderer_core::GraphTexture;

use crate::feature::lighting::lights::lights_feature::LightsFeature;
use crate::scene::level::light_shadow_atlas::LightShadowAtlas;

/// What the light shafts read of the units before them: the view's lights and the scene's light shadow atlas their
/// volumes march through, and the scene the reflections keep, none where they keep none.
#[derive(Clone, Copy)]
pub struct ShaftInputs<'a> {
  pub lights: &'a LightsFeature,
  pub atlas: LightShadowAtlas<'a>,
  pub reflected: Option<GraphTexture>,
}
