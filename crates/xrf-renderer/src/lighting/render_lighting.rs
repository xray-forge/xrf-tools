use glam::Vec3;
use xrf_math::EPS;

use crate::lighting::sun_direction::to_renderer_sun_direction;

/// `ps_r2_gloss_factor`, the engine's default.
const GLOSS_FACTOR: f32 = 4.0;

/// The floor `phase_combine` keeps ambient above.
const MINIMUM_AMBIENT: f32 = 0.001;

/// What a scene is lit by, in the terms a weather keyframe uses.
#[derive(Clone, Copy, Debug, PartialEq)]
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
}

impl RenderLighting {
  /// The sun direction the passes bind: normalised, a zero one left zero since it points nowhere.
  pub fn get_sun_direction(&self) -> Vec3 {
    self.sun_direction.normalize_or_zero()
  }

  /// `Ldynamic_color.w`: what the sun contributes to specular, `u_diffuse2s` of its colour (`r2_types.h`).
  pub fn get_sun_specular(&self) -> f32 {
    let mean: f32 = self.sun_color.element_sum() / 3.0;

    GLOSS_FACTOR * if mean < 1.0 { mean.powf(2.0 / 3.0) } else { mean }
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
    }
  }
}
