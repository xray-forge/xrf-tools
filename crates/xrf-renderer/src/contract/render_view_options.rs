use serde::{Deserialize, Serialize};

use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_asset_lighting::RenderAssetLighting;
use crate::contract::render_backdrop_squares::RenderBackdropSquares;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_light_scales::RenderLightScales;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_lod_settings::RenderLodSettings;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_spawn_category::RenderSpawnCategory;
use crate::contract::render_sun_shafts::RenderSunShafts;
use crate::contract::render_surface_color::RenderSurfaceColor;
use crate::contract::render_upscaling_settings::RenderUpscalingSettings;
use crate::contract::render_water_settings::RenderWaterSettings;

/// What one viewport draws its scene with, as its viewer's toolbar sets it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewOptions {
  /// Whether the scene is lit, else shown as its raw albedo.
  pub is_lit: bool,
  /// Whether every static surface draws as its triangles' edges.
  pub is_wireframe: bool,
  /// What colour surfaces' albedo is drawn with: their textures, clay, or their shader's tint.
  pub surface_color: RenderSurfaceColor,
  /// Whether bump textures bend the normal.
  pub is_bumped: bool,
  /// How far the baked hemisphere darkens the ambient: zero for not at all.
  pub hemi_strength: f32,
  /// Whether what the last frame's depth hides is left undrawn.
  pub is_occlusion_culled: bool,
  /// How much of the static geometry draws at a distance.
  pub lod: RenderLodSettings,
  /// An asset viewer's light, in place of the weather's; none for a level.
  pub asset_lighting: Option<RenderAssetLighting>,
  /// What shows where nothing was drawn and neither the sky nor the fog is, each channel zero to one; none for the
  /// level viewer's own.
  pub backdrop: Option<[f32; 3]>,
  /// The backdrop laid out as a checkerboard with a second colour, as behind a picture with alpha; none for a plain one.
  pub backdrop_squares: Option<RenderBackdropSquares>,
  /// Times a uv checker repeats over a surface's base coordinate, drawn in place of its textures; zero for none.
  pub checker: f32,
  /// The colour a surface naming no base texture is drawn, each channel zero to one; none for white.
  pub plain_color: Option<[f32; 3]>,
  /// Whether surfaces cut out and blend as their shaders ask, or draw solid.
  pub is_alpha_visible: bool,
  /// Whether the weather's fog hides the distance.
  pub is_fogged: bool,
  /// Whether the weather's sky is drawn behind the level, rather than a plain backdrop.
  pub is_sky_visible: bool,
  /// Whether the distance fades into the sky's haze rather than into the sky itself.
  pub is_sky_hazed: bool,
  /// Whether the weather's clouds cross the sky.
  pub is_clouded: bool,
  /// Whether the sun's lens flares are drawn over the frame, `disable_lens_flare 0`; its sprite and gradient are drawn
  /// either way.
  pub is_lens_flared: bool,
  /// Whether the sun's light shafts are drawn through its shadow, `r2_sun_shafts` (`r2_sunshafts_mode volumetric` on
  /// Monolith) at its highest quality.
  pub is_sun_shafted: bool,
  /// How finely they step, and Monolith's floor under their density.
  pub sun_shafts: RenderSunShafts,
  /// Whether the weather's rain falls and wets surfaces.
  pub is_rainy: bool,
  /// Whether the weather's bolts strike.
  pub is_thundering: bool,
  /// Whether the weather's wind sways trees and grass.
  pub is_windy: bool,
  /// Whether the level's wall marks are laid over its surfaces.
  pub is_wallmarked: bool,
  /// Whether the level's particle systems play and draw.
  pub is_particled: bool,
  /// Whether its campfires burn, as `CZoneCampfire` starts, rather than smoulder out.
  pub is_campfire_lit: bool,
  /// Whether the weather's ambient effects play near the camera and bring their wind.
  pub is_ambient_played: bool,
  /// Which groups of the level's spawned objects are drawn.
  pub is_spawned_props: bool,
  pub is_spawned_items: bool,
  pub is_spawned_weapons: bool,
  pub is_spawned_lamps: bool,
  /// Whether the spawned objects a new game releases are drawn too, each with its group.
  pub is_spawned_released: bool,
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
  /// Device pixels the scene is drawn tall before its render scale, or `None` for the viewport's own; one taller than
  /// the viewport draws at the viewport's.
  pub render_height: Option<u32>,
  /// How the game's console scales the sun, the hemisphere and the ambient.
  pub light_scales: RenderLightScales,
  /// What the finished frame is corrected by.
  pub corrections: RenderImageCorrections,
}

impl Default for RenderViewOptions {
  fn default() -> Self {
    Self {
      is_lit: true,
      is_wireframe: false,
      surface_color: RenderSurfaceColor::Textured,
      is_bumped: true,
      hemi_strength: 1.0,
      is_occlusion_culled: true,
      lod: RenderLodSettings::default(),
      asset_lighting: None,
      backdrop: None,
      backdrop_squares: None,
      checker: 0.0,
      plain_color: None,
      is_alpha_visible: true,
      is_fogged: true,
      is_sky_visible: true,
      is_sky_hazed: false,
      is_clouded: true,
      is_lens_flared: true,
      is_sun_shafted: true,
      sun_shafts: RenderSunShafts::default(),
      is_rainy: true,
      is_thundering: true,
      is_windy: true,
      is_wallmarked: true,
      is_particled: true,
      is_campfire_lit: true,
      is_ambient_played: true,
      is_spawned_props: true,
      is_spawned_items: true,
      is_spawned_weapons: true,
      is_spawned_lamps: true,
      is_spawned_released: false,
      exposure: RenderExposureSettings::default(),
      shadows: RenderShadowSettings::default(),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      lights: RenderLightsSettings::default(),
      water: RenderWaterSettings::default(),
      grass: RenderGrassSettings::default(),
      debug_view: RenderDebugView::Final,
      antialiasing: RenderAntialiasing::None,
      upscaling: RenderUpscalingSettings::default(),
      render_height: None,
      light_scales: RenderLightScales::default(),
      corrections: RenderImageCorrections::default(),
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
