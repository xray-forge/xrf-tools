use serde::{Deserialize, Serialize};

use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_bloom_settings::RenderBloomSettings;
use crate::contract::render_debanding_settings::RenderDebandingSettings;
use crate::contract::render_display_transform::RenderDisplayTransform;
use crate::contract::render_enhanced_bloom_settings::RenderEnhancedBloomSettings;
use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::contract::render_fog_settings::RenderFogSettings;
use crate::contract::render_glass_settings::RenderGlassSettings;
use crate::contract::render_grading_settings::RenderGradingSettings;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_indirect_light_settings::RenderIndirectLightSettings;
use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_light_model::RenderLightModel;
use crate::contract::render_light_scales::RenderLightScales;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_lod_settings::RenderLodSettings;
use crate::contract::render_parallax_settings::RenderParallaxSettings;
use crate::contract::render_rain_settings::RenderRainSettings;
use crate::contract::render_reflection_settings::RenderReflectionSettings;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_sun_shafts::RenderSunShafts;
use crate::contract::render_terrain_settings::RenderTerrainSettings;
use crate::contract::render_tonemap_mode::RenderTonemapMode;
use crate::contract::render_volumetric_settings::RenderVolumetricSettings;
use crate::contract::render_water_settings::RenderWaterSettings;

/// Each of the renderer's features as a view sets it, a full set a view: one struct a feature, owned by its module.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewFeatures {
  pub exposure: RenderExposureSettings,
  /// Where the frame is tonemapped: in combine and each forward pass, or once by the window pass over linear light.
  pub tonemap: RenderTonemapMode,
  /// What the light is computed in: the engine's display-encoded values, or linear light.
  pub light_model: RenderLightModel,
  /// How bright the physical light's sources stand against each other and the day.
  pub light_level: RenderLightLevel,
  /// How much of the weather's own sun colour the physical light levels keep in the sun's hue: nought the atmosphere's
  /// alone, one the weather's.
  pub sun_tint: f32,
  /// What shows the physical light's frame on the display.
  pub display_transform: RenderDisplayTransform,
  /// How the physical light's frame is graded before its display transform.
  pub grading: RenderGradingSettings,
  pub bloom: RenderBloomSettings,
  /// Which bloom is drawn, and the enhanced bloom's strengths.
  pub enhanced_bloom: RenderEnhancedBloomSettings,
  /// How the fog is drawn beyond the weather's keys: the engine's distance fog, or the enhanced height fog.
  pub fog: RenderFogSettings,
  /// Whether the sky's colour bands are smoothed.
  pub debanding: RenderDebandingSettings,
  pub shadows: RenderShadowSettings,
  pub ambient_occlusion: RenderAmbientOcclusionSettings,
  /// The light the frame's surfaces bounce onto each other, searched as the ambient occlusion's settings describe.
  pub indirect_light: RenderIndirectLightSettings,
  /// What glossy surfaces reflect: the sky's cube, or what the frame shows where a ray finds it.
  pub reflections: RenderReflectionSettings,
  /// How rain falls and wets the surfaces: the engine's streaks or refracting drops, and the engine's splashes or wet
  /// surfaces and puddles that build up and dry.
  pub rain: RenderRainSettings,
  pub lights: RenderLightsSettings,
  pub water: RenderWaterSettings,
  /// How glass is drawn: the engine's, or physically based.
  pub glass: RenderGlassSettings,
  pub grass: RenderGrassSettings,
  /// How finely the sun's shafts step, and Monolith's floor under their density.
  pub sun_shafts: RenderSunShafts,
  /// How the light the air scatters is drawn: the engine's shafts, or a froxel volume.
  pub volumetric: RenderVolumetricSettings,
  /// How much of the static geometry draws at a distance.
  pub lod: RenderLodSettings,
  /// How the frame's edges are smoothed.
  pub antialiasing: RenderAntialiasing,
  /// How the level's surfaces are displaced by their height.
  pub parallax: RenderParallaxSettings,
  /// How the terrain's detail textures repeat across the ground.
  pub terrain: RenderTerrainSettings,
  /// What the finished frame is corrected by.
  pub corrections: RenderImageCorrections,
  /// How the game's console scales the sun, the hemisphere and the ambient.
  pub light_scales: RenderLightScales,
  /// How far the baked hemisphere darkens the ambient: zero for not at all.
  pub hemi_strength: f32,
  /// Whether what the last frame's depth hides is left undrawn.
  pub is_occlusion_culled: bool,
  /// Whether the opaque surfaces' depth is drawn alone ahead of their G-buffer draws, so each pixel is shaded once.
  pub is_depth_prepassed: bool,
}

impl Default for RenderViewFeatures {
  fn default() -> Self {
    Self {
      exposure: RenderExposureSettings::default(),
      tonemap: RenderTonemapMode::Engine,
      light_model: RenderLightModel::Engine,
      light_level: RenderLightLevel::Authored,
      sun_tint: 0.0,
      display_transform: RenderDisplayTransform::Engine,
      grading: RenderGradingSettings::default(),
      bloom: RenderBloomSettings::default(),
      enhanced_bloom: RenderEnhancedBloomSettings::default(),
      fog: RenderFogSettings::default(),
      debanding: RenderDebandingSettings::default(),
      shadows: RenderShadowSettings::default(),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      indirect_light: RenderIndirectLightSettings::default(),
      reflections: RenderReflectionSettings::default(),
      rain: RenderRainSettings::default(),
      lights: RenderLightsSettings::default(),
      water: RenderWaterSettings::default(),
      glass: RenderGlassSettings::default(),
      grass: RenderGrassSettings::default(),
      sun_shafts: RenderSunShafts::default(),
      volumetric: RenderVolumetricSettings::default(),
      lod: RenderLodSettings::default(),
      antialiasing: RenderAntialiasing::None,
      parallax: RenderParallaxSettings::default(),
      terrain: RenderTerrainSettings::default(),
      corrections: RenderImageCorrections::default(),
      light_scales: RenderLightScales::default(),
      hemi_strength: 1.0,
      is_occlusion_culled: true,
      is_depth_prepassed: false,
    }
  }
}
