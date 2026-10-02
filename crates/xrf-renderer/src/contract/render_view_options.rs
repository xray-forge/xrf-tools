use serde::{Deserialize, Serialize};

use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_spawn_category::RenderSpawnCategory;
use crate::contract::render_upscaling_settings::RenderUpscalingSettings;
use crate::contract::render_water_settings::RenderWaterSettings;

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
  /// Whether the weather's fog hides the distance.
  pub is_fogged: bool,
  /// Whether the weather's sky is drawn behind the level, rather than a plain backdrop.
  pub is_sky_visible: bool,
  /// Whether the distance fades into the sky's haze rather than into the sky itself.
  pub is_sky_hazed: bool,
  /// Whether the weather's clouds cross the sky.
  pub is_clouded: bool,
  /// Whether the weather's rain falls and wets surfaces.
  pub is_rainy: bool,
  /// Whether the weather's bolts strike.
  pub is_thundering: bool,
  /// Whether the weather's wind sways trees and grass.
  pub is_windy: bool,
  /// Whether the level's wall marks are laid over its surfaces.
  pub is_wallmarked: bool,
  /// Which groups of the level's spawned objects are drawn.
  pub is_spawned_props: bool,
  pub is_spawned_items: bool,
  pub is_spawned_weapons: bool,
  pub is_spawned_lamps: bool,
  pub exposure: RenderExposureSettings,
  pub shadows: RenderShadowSettings,
  pub ambient_occlusion: RenderAmbientOcclusionSettings,
  pub lights: RenderLightsSettings,
  pub water: RenderWaterSettings,
  pub grass: RenderGrassSettings,
  /// Which picture the viewport shows: its frame, or one of the targets the frame was built from.
  pub debug_view: RenderDebugView,
  /// How the frame's edges are smoothed.
  pub antialiasing: RenderAntialiasing,
  /// What the scene is drawn at, and how its upscaled frame is sharpened.
  pub upscaling: RenderUpscalingSettings,
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
      is_fogged: true,
      is_sky_visible: true,
      is_sky_hazed: false,
      is_clouded: true,
      is_rainy: true,
      is_thundering: true,
      is_windy: true,
      is_wallmarked: true,
      is_spawned_props: true,
      is_spawned_items: true,
      is_spawned_weapons: true,
      is_spawned_lamps: true,
      exposure: RenderExposureSettings::default(),
      shadows: RenderShadowSettings::default(),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      lights: RenderLightsSettings::default(),
      water: RenderWaterSettings::default(),
      grass: RenderGrassSettings::default(),
      debug_view: RenderDebugView::Final,
      antialiasing: RenderAntialiasing::None,
      upscaling: RenderUpscalingSettings::default(),
    }
  }
}

impl RenderViewOptions {
  /// Whether a group of spawned objects is drawn.
  pub fn is_spawned(&self, category: RenderSpawnCategory) -> bool {
    match category {
      RenderSpawnCategory::Props => self.is_spawned_props,
      RenderSpawnCategory::Items => self.is_spawned_items,
      RenderSpawnCategory::Weapons => self.is_spawned_weapons,
      RenderSpawnCategory::Lamps => self.is_spawned_lamps,
    }
  }
}
