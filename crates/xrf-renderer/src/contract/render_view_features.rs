use serde::{Deserialize, Serialize};

use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_bloom_settings::RenderBloomSettings;
use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_light_scales::RenderLightScales;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_lod_settings::RenderLodSettings;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_sun_shafts::RenderSunShafts;
use crate::contract::render_water_settings::RenderWaterSettings;

/// Each of the renderer's features as a view sets it, a full set a view: one struct a feature, owned by its module.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewFeatures {
  pub exposure: RenderExposureSettings,
  pub bloom: RenderBloomSettings,
  pub shadows: RenderShadowSettings,
  pub ambient_occlusion: RenderAmbientOcclusionSettings,
  pub lights: RenderLightsSettings,
  pub water: RenderWaterSettings,
  pub grass: RenderGrassSettings,
  /// How finely the sun's shafts step, and Monolith's floor under their density.
  pub sun_shafts: RenderSunShafts,
  /// How much of the static geometry draws at a distance.
  pub lod: RenderLodSettings,
  /// How the frame's edges are smoothed.
  pub antialiasing: RenderAntialiasing,
  /// What the finished frame is corrected by.
  pub corrections: RenderImageCorrections,
  /// How the game's console scales the sun, the hemisphere and the ambient.
  pub light_scales: RenderLightScales,
  /// How far the baked hemisphere darkens the ambient: zero for not at all.
  pub hemi_strength: f32,
  /// Whether what the last frame's depth hides is left undrawn.
  pub is_occlusion_culled: bool,
}

impl Default for RenderViewFeatures {
  fn default() -> Self {
    Self {
      exposure: RenderExposureSettings::default(),
      bloom: RenderBloomSettings::default(),
      shadows: RenderShadowSettings::default(),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      lights: RenderLightsSettings::default(),
      water: RenderWaterSettings::default(),
      grass: RenderGrassSettings::default(),
      sun_shafts: RenderSunShafts::default(),
      lod: RenderLodSettings::default(),
      antialiasing: RenderAntialiasing::None,
      corrections: RenderImageCorrections::default(),
      light_scales: RenderLightScales::default(),
      hemi_strength: 1.0,
      is_occlusion_culled: true,
    }
  }
}
