use serde::{Deserialize, Serialize};

use crate::contract::render_water_mode::RenderWaterMode;

/// The water (`water.vs`, `water.ps`, `waterd.ps`): rippled and reflecting the sky, blended over the depth behind it
/// and distorting it. The engine's own look by default: its constants are `shared/waterconfig.h`'s and `def_distort`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderWaterSettings {
  /// Off, what lies under the water shows.
  pub is_enabled: bool,
  /// The engine's water, or Screen Space Shaders' water, which the strengths below the distortion's shape.
  pub mode: RenderWaterMode,
  /// `r2_soft_water`: soft water fades by the depth behind it, darkens with it and lays foam in the shallows.
  pub is_soft: bool,
  /// Whether water writes the distortion it causes, moving what is seen through it.
  pub is_distorted: bool,
  /// How high the waves lift the surface, in metres: `W_POSITION_SHIFT_HEIGHT`.
  pub wave_height: f32,
  /// How fast they run: `W_POSITION_SHIFT_SPEED`.
  pub wave_speed: f32,
  /// What the two normal layers' scroll is multiplied by, one as the engine scrolls them.
  pub ripple: f32,
  /// What the sky's reflection is multiplied by, one as the engine mixes it.
  pub reflection: f32,
  /// How far the distortion target moves what is behind it, a share of the screen: `def_distort`, which moves what
  /// the distorting particles write as well.
  pub distortion: f32,
  /// The enhanced water's refraction: how far its waves move what lies under it, `ssfx_water_setup1.x`.
  pub refraction: f32,
  /// How deep the enhanced water clears before it clouds into its colour, `ssfx_water_setup1.y`.
  pub turbidity: f32,
  /// Metres of depth over which the enhanced water's edge fades into what lies under it, `ssfx_water_setup1.z`.
  pub soft_border: f32,
  /// How much of its reflection the enhanced water shows by the fresnel, `ssfx_water_setup2.x`; none draws no reflection.
  pub reflectivity: f32,
  /// How far the enhanced water's reflection is blurred, `ssfx_water.y`.
  pub reflection_blur: f32,
  /// How much noise mixes the clear reflection into the blurred one, `ssfx_water.z`.
  pub blur_noise: f32,
  /// How bright the sun's highlight on the enhanced water is, `ssfx_water_setup2.y`.
  pub specular: f32,
  /// How bright the light the enhanced water gathers onto its bottom is, `ssfx_water_setup2.z`.
  pub caustics: f32,
  /// How high the enhanced water's waves stand in its parallax, `ssfx_water_setup1.w`; none draws it flat.
  pub parallax_height: f32,
  /// How strongly rain ripples the enhanced water, `ssfx_water_setup2.w`; none skips them.
  pub ripples: f32,
}

impl Default for RenderWaterSettings {
  /// The engine's own water.
  fn default() -> Self {
    Self {
      is_enabled: true,
      mode: RenderWaterMode::Engine,
      is_soft: true,
      is_distorted: true,
      wave_height: 1.0 / 60.0,
      wave_speed: 25.0,
      ripple: 1.0,
      reflection: 1.0,
      distortion: 0.05,
      refraction: 0.6,
      turbidity: 3.0,
      soft_border: 0.3,
      reflectivity: 0.8,
      reflection_blur: 0.8,
      blur_noise: 1.0,
      specular: 6.0,
      caustics: 0.3,
      parallax_height: 0.05,
      ripples: 0.5,
    }
  }
}
