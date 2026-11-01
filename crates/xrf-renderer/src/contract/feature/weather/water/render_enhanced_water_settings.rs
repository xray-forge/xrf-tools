use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

/// The enhanced water's strengths: what lies under it refracted, clouded with depth, the scene reflected,
/// bordered softly.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderEnhancedWaterSettings {
  /// How far its waves move what lies under it.
  #[setting(min = 0, max = 2)]
  pub refraction: f32,
  /// How deep it clears before it clouds into its colour.
  #[setting(min = 0, max = 10)]
  pub turbidity: f32,
  /// Metres of depth over which its edge fades into what lies under it.
  #[setting(min = 0, max = 1)]
  pub soft_border: f32,
  /// How much of its reflection it shows by the fresnel; none draws no reflection.
  #[setting(min = 0, max = 1)]
  pub reflectivity: f32,
  /// How far its reflection is blurred.
  #[setting(min = 0, max = 1)]
  pub reflection_blur: f32,
  /// How much noise mixes the clear reflection into the blurred one.
  #[setting(min = 0, max = 1)]
  pub blur_noise: f32,
  /// How much of its colour the reflection keeps, one all of it, none grey.
  #[setting(min = 0, max = 1)]
  pub vibrance: f32,
  /// How bright the sun's highlight on it is.
  #[setting(min = 0, max = 10)]
  pub specular: f32,
  /// How bright the light it gathers onto its bottom is.
  #[setting(min = 0, max = 1)]
  pub caustics: f32,
  /// How high its waves stand in its parallax; none draws it flat.
  #[setting(min = 0, max = 0.1)]
  pub parallax_height: f32,
  /// How strongly rain ripples it; none skips them.
  #[setting(min = 0, max = 1)]
  pub ripples: f32,
  /// What every scroll of its maps is multiplied by, one as designed; none stills it.
  #[setting(min = 0, max = 2)]
  pub flow: f32,
  /// How much of its pace it keeps in still air, one as designed; none stills it while no
  /// wind blows.
  #[setting(min = 0, max = 1)]
  pub calm_flow: f32,
  /// How far it breaks its maps' repeat: its second layer tiled apart from the first, a broad layer faded in with
  /// distance, and its colour's read bent and mixed with a second; none draws a single repeat.
  #[setting(min = 0, max = 1)]
  pub variation: f32,
}

impl Default for RenderEnhancedWaterSettings {
  /// Its designed strengths, but for caustics a sixth as bright, calm water flowing a fifth as fast, its maps' repeat
  /// broken, and its reflection at its full colour rather than the designed 0.6.
  fn default() -> Self {
    Self {
      refraction: 0.6,
      turbidity: 3.0,
      soft_border: 0.05,
      reflectivity: 0.8,
      reflection_blur: 0.8,
      blur_noise: 1.0,
      vibrance: 1.0,
      specular: 6.0,
      caustics: 0.05,
      parallax_height: 0.05,
      ripples: 0.5,
      flow: 1.0,
      calm_flow: 0.2,
      variation: 0.75,
    }
  }
}
