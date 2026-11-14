use glam::Vec3;
use xrf_level::LevelLight;

/// One light of `build.lights` as the compilers light with it (`R_Light`, `xrLC_Light/R_light.h`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightSource {
  /// `LT_DIRECT`: lit from `direction`, from nowhere in particular; any other kind is lit from `position`.
  pub(crate) is_directional: bool,
  /// `LT_POINT`: dropped for a place out of its reach; a light of any other kind reaches everywhere.
  pub is_point: bool,
  pub diffuse: Vec3,
  pub position: Vec3,
  /// The way the light travels; a ray toward it runs the other way.
  pub direction: Vec3,
  pub range: f32,
  pub range_squared: f32,
  /// Constant, linear and quadratic.
  pub attenuation: [f32; 3],
  /// What xrLC takes off a static point light's colour with distance, so it ends at its range (`R_Light::falloff`).
  pub falloff: f32,
  pub energy: f32,
}

impl LightSource {
  /// Whether the light reaches the sphere centred at `center` of radius `radius`: every light but a point one whose
  /// range ends short of it (`base_lighting::select`, `xrDO_Light/base_lighting.cpp`).
  pub fn reaches(&self, center: Vec3, radius: f32) -> bool {
    !self.is_point || center.distance(self.position) <= radius + self.range
  }
}

impl From<&LevelLight> for LightSource {
  fn from(light: &LevelLight) -> Self {
    Self {
      is_directional: light.is_directional(),
      is_point: light.is_point(),
      diffuse: Vec3::from(&light.diffuse),
      position: Vec3::from(&light.position),
      direction: Vec3::from(&light.direction),
      range: light.range,
      range_squared: light.range_squared,
      attenuation: [
        light.attenuation_constant,
        light.attenuation_linear,
        light.attenuation_quadratic,
      ],
      falloff: light.falloff,
      energy: light.energy,
    }
  }
}
