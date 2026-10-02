use glam::{Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_math::EPS;

use crate::lighting::light_specular::to_light_specular;
use crate::lighting::render_clouds::RenderClouds;
use crate::lighting::render_fog::RenderFog;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::lighting::render_sky::RenderSky;
use crate::lighting::render_thunderbolt_strike::RenderThunderboltStrike;
use crate::lighting::render_tree_wind::RenderTreeWind;
use crate::lighting::sun_direction::to_renderer_sun_direction;

/// The floor `phase_combine` keeps ambient above.
const MINIMUM_AMBIENT: f32 = 0.001;

/// The sky `default_clear` names at noon, and its irradiance cube.
const DEFAULT_SKY: &str = "sky\\sky_7_cube";
const DEFAULT_SKY_ENVIRONMENT: &str = "sky\\sky_7_cube#small";

/// What a scene is lit by, in the terms a weather keyframe uses.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderLighting {
  /// The direction sunlight travels, in renderer space, of length one.
  pub sun_direction: Vec3,
  /// `sun_color`.
  pub sun_color: Vec3,
  /// `hemisphere_color`.
  pub hemisphere_color: Vec3,
  /// `ambient_color`.
  pub ambient_color: Vec3,
  /// What the sky's irradiance cube returns, standing in for it while it is not up.
  pub sky_irradiance: Vec3,
  /// Distance fog, or none.
  pub fog: Option<RenderFog>,
  pub sky: RenderSky,
  /// How the trees sway, or none for trees standing still.
  pub trees: Option<RenderTreeWind>,
  /// `water_intensity`: how bright the depth of soft water and its foam are, one by a clear day.
  pub water_intensity: f32,
  /// How hard it rains, or none for a dry sky.
  pub rain: Option<RenderRainfall>,
  /// Whose shaders the scene is drawn by: its sky, and its surfaces' reflections.
  pub engine: XrayEngine,
  /// The bolt striking this frame, or none.
  pub thunderbolt: Option<RenderThunderboltStrike>,
}

impl RenderLighting {
  /// The sun direction the passes bind: normalised, a zero one left zero since it points nowhere.
  pub fn get_sun_direction(&self) -> Vec3 {
    self.sun_direction.normalize_or_zero()
  }

  /// `Ldynamic_color.w`: what the sun contributes to specular, `u_diffuse2s` of its colour (`r2_types.h`).
  pub fn get_sun_specular(&self) -> f32 {
    to_light_specular(self.sun_color)
  }

  /// `L_ambient`.
  pub fn get_ambient(&self) -> Vec3 {
    (self.ambient_color * 2.0).max(Vec3::splat(MINIMUM_AMBIENT))
  }

  /// `env_color.rgb` as combine binds it.
  ///
  /// `CEnvDescriptorMixer::lerp` adds `EPS` so a black hemisphere is never exactly zero, then `phase_combine` doubles it.
  pub fn get_environment(&self) -> Vec3 {
    (self.hemisphere_color * 2.0 + EPS) * 2.0
  }
}

impl Default for RenderLighting {
  /// Noon of `default_clear` without its fog; the sky irradiance is the measured mean of that keyframe's
  /// `sky_7_cube#small`.
  fn default() -> Self {
    Self {
      sun_direction: to_renderer_sun_direction(-68.999985, -30.0),
      sun_color: Vec3::new(0.905882, 0.839216, 0.694118),
      hemisphere_color: Vec3::new(0.470588, 0.368627, 0.329412),
      ambient_color: Vec3::splat(0.02),
      sky_irradiance: Vec3::new(0.5, 0.511, 0.548),
      fog: None,
      sky: RenderSky {
        textures: [Some(DEFAULT_SKY.to_owned()), Some(DEFAULT_SKY.to_owned())],
        environments: [
          Some(DEFAULT_SKY_ENVIRONMENT.to_owned()),
          Some(DEFAULT_SKY_ENVIRONMENT.to_owned()),
        ],
        blend: 0.0,
        color: Vec3::splat(0.851001),
        rotation: 0.0,
        clouds: RenderClouds {
          textures: [None, None],
          color: Vec4::ZERO,
          rotation: 0.0,
        },
      },
      trees: Some(RenderTreeWind::default()),
      water_intensity: 1.0,
      rain: None,
      engine: XrayEngine::Vanilla,
      thunderbolt: None,
    }
  }
}
