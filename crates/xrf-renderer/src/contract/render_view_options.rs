use serde::{Deserialize, Serialize};

use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_shadow_settings::RenderShadowSettings;

/// What one viewport draws its scene with, as its viewer's toolbar sets it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewOptions {
  /// Whether the scene is lit, else shown as its raw albedo.
  pub is_lit: bool,
  /// Whether surfaces wear their textures, else their flat colours.
  pub is_textured: bool,
  /// Whether bump textures bend the normal.
  pub is_bumped: bool,
  /// How far the baked hemisphere darkens the ambient: zero for not at all.
  pub hemi_strength: f32,
  /// Whether what the last frame's depth hides is left undrawn.
  pub is_occlusion_culled: bool,
  /// Whether distant trees are drawn as their impostors.
  pub is_impostors: bool,
  /// `r__geometry_lod`: every screen area threshold scales with it.
  pub geometry_lod: f32,
  /// What the tonemap multiplies by before the exposure's own scale.
  pub tonemap_scale: f32,
  pub exposure: RenderExposureSettings,
  pub shadows: RenderShadowSettings,
  pub ambient_occlusion: RenderAmbientOcclusionSettings,
  pub lights: RenderLightsSettings,
}

impl Default for RenderViewOptions {
  fn default() -> Self {
    Self {
      is_lit: true,
      is_textured: true,
      is_bumped: true,
      hemi_strength: 1.0,
      is_occlusion_culled: true,
      is_impostors: true,
      geometry_lod: 0.75,
      tonemap_scale: 1.0,
      exposure: RenderExposureSettings::default(),
      shadows: RenderShadowSettings::default(),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      lights: RenderLightsSettings::default(),
    }
  }
}
