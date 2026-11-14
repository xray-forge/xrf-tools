use glam::Vec3;
use xrf_math::Vector3d;
use xrf_ogf::OgfTreeColor;

/// A tree placement's terms for the static lights' colour, the sky and the sun, as its `c_scale` and `c_bias` hold
/// them (the engine's `_5color`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightTreeColor {
  pub rgb: Vec3,
  pub hemi: f32,
  pub sun: f32,
}

impl From<LightTreeColor> for OgfTreeColor {
  fn from(color: LightTreeColor) -> Self {
    Self {
      rgb: Vector3d::new(color.rgb.x, color.rgb.y, color.rgb.z),
      hemi: color.hemi,
      sun: color.sun,
    }
  }
}
