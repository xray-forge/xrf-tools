use std::str::FromStr;

use serde::{Deserialize, Serialize};
use xrf_renderer::{
  RENDER_MAX_CONTACT_SHADOW_LIGHTS, RENDER_MAX_SHADOW_FILTER, RenderAmbientOcclusionMethod, RenderAntialiasing,
  RenderBloomMode, RenderContactShadowMode, RenderDebandingMode, RenderFogMode, RenderFoliageMode, RenderGlassMode,
  RenderIndirectLightMode, RenderLightShadowFilter, RenderParallaxMode, RenderRainDropsMode, RenderRainMode,
  RenderReflectionMode, RenderReflectionProbe, RenderTerrainTiling, RenderTextureFiltering, RenderTonemapMode,
  RenderTraceSize, RenderViewFeatures, RenderVolumetricMode, RenderWaterMode,
};

use crate::game_settings::GameSettings;

/// A named starting point for the game's settings, which its settings file and arguments change on top of.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GamePreset {
  /// Every enhanced mode at a high quality, short of ray reconstruction and traced occlusion.
  #[default]
  High,
}

/// A preset by its name as a settings file or an argument writes it, in any case.
impl FromStr for GamePreset {
  type Err = String;

  fn from_str(name: &str) -> Result<Self, String> {
    Self::ALL
      .into_iter()
      .find(|preset| preset.get_name().eq_ignore_ascii_case(name.trim()))
      .ok_or_else(|| {
        let names: Vec<&str> = Self::ALL.into_iter().map(Self::get_name).collect();

        format!("the game has no preset '{name}': it has {}", names.join(", "))
      })
  }
}

impl GamePreset {
  /// Every preset, lowest first.
  pub const ALL: [Self; 1] = [Self::High];

  /// Its name as a settings file and an argument write it.
  pub fn get_name(self) -> &'static str {
    match self {
      Self::High => "high",
    }
  }

  /// What the preset sets every setting to, over the renderer's own defaults.
  pub fn get_settings(self) -> GameSettings {
    match self {
      Self::High => Self::get_high(),
    }
  }

  fn get_high() -> GameSettings {
    let mut settings: GameSettings = GameSettings::default();
    let defaults: RenderViewFeatures = RenderViewFeatures::default();
    let view: &mut RenderViewFeatures = &mut settings.view;

    // VBAO over XeGTAO, which lets light through behind grass and fences.
    view.ambient_occlusion.method = RenderAmbientOcclusionMethod::Vbao;
    view.ambient_occlusion.gtao.slices = 4;
    view.ambient_occlusion.gtao.steps = 3;
    view.ambient_occlusion.rtao.rays = 2;
    view.ambient_occlusion.vbao.steps = 8;
    // The temporal resolve the cut-outs' hashed alpha and the jitter are for: DLSS at the window's size, which a GPU
    // or build without it resolves with FSR.
    view.antialiasing = RenderAntialiasing::Dlss;
    // The sky's bands smoothed in its own textures, which leaves the rest of the frame as drawn.
    view.debanding.mode = RenderDebandingMode::Enhanced;
    view.enhanced_bloom.mode = RenderBloomMode::Enhanced;
    view.fog.mode = RenderFogMode::Enhanced;
    view.glass.mode = RenderGlassMode::Enhanced;
    // The enhanced foliage motion, each tuft stood along its ground's slope, planted to 300 m and casting its own
    // shadows in the sun's three nearest cascades out to 75 m.
    view.grass.foliage.mode = RenderFoliageMode::Enhanced;
    view.grass.is_aligned = true;
    view.grass.is_shadowed = true;
    view.grass.radius = 300.0;
    view.grass.shadow_cascades = 3;
    view.grass.shadow_radius = 75.0;
    view.indirect_light.mode = RenderIndirectLightMode::Enhanced;
    view.indirect_light.steps = 2;
    // The opaque surfaces' depth drawn first, so a pixel under stacked walls, decks and trunks is shaded once.
    view.is_depth_prepassed = true;
    view.is_occlusion_culled = true;
    // Anomaly's soft shadows for the lamps.
    view.lights.shadow_filter = RenderLightShadowFilter::Soft;
    // Trees stand in as impostors twice as far as the engine's: a clump's screen area falls with its distance squared
    // and the thresholds are squared, so half the engine's `r_ssaLOD_A` and `r_ssaLOD_B`.
    view.lod.ssa_a /= 2.0;
    view.lod.ssa_b /= 2.0;
    view.parallax.mode = RenderParallaxMode::Enhanced;
    view.parallax.steps = 32;
    // Wet surfaces that build up and dry, puddles, and drops bending the scene behind them.
    view.rain.mode = RenderRainMode::Enhanced;
    view.rain.drops = RenderRainDropsMode::Enhanced;
    // Traced at the frame's size up to the default roughness, and where the GPU traces rays every frame, the grass
    // reflected too.
    view.reflections.mode = RenderReflectionMode::Enhanced;
    view.reflections.is_grass_reflected = true;
    view.reflections.is_traced = true;
    view.reflections.probe = RenderReflectionProbe::Finest;
    view.reflections.steps = 96;
    view.reflections.trace_interval = 1;
    view.reflections.trace_roughness = defaults.reflections.trace_roughness;
    view.reflections.trace_size = RenderTraceSize::Full;
    view.shadows.contact.mode = RenderContactShadowMode::Enhanced;
    view.shadows.contact.lights = RENDER_MAX_CONTACT_SHADOW_LIGHTS;
    view.shadows.contact.steps = 32;
    view.shadows.filter = RENDER_MAX_SHADOW_FILTER;
    view.shadows.resolution = 4096;
    // The terrain's details hex tiled, so no repeat of the ground lines up with the next.
    view.terrain.tiling = RenderTerrainTiling::Varied;
    // The physical light levels' sun in the atmosphere's colour alone.
    view.sun_tint = 0.0;
    // The scene held as linear light and tonemapped once at the viewport's size, as an upscaler wants it.
    view.tonemap = RenderTonemapMode::Enhanced;
    // The sun and every lamp scattered in one froxel volume, in the engine's shafts' place, out to 200 m.
    view.volumetric.mode = RenderVolumetricMode::Enhanced;
    view.volumetric.distance = 200.0;
    view.water.mode = RenderWaterMode::Enhanced;

    settings.render.texture_filtering = RenderTextureFiltering::Anisotropic16;
    // The engine's full quality, `texture_lod 0`, as Anomaly's extreme preset sets it.
    settings.render.texture_lod = 0;

    settings
  }
}
