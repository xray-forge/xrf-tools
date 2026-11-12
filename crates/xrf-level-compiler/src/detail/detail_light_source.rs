use glam::Vec3;
use xrf_level::LevelLight;

/// One light of `build.lights` as the detail lighter uses it (`R_Light`, `xrLC_Light/R_light.h`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DetailLightSource {
  /// `LT_DIRECT`: lit from `direction`, from nowhere in particular; any other kind is lit from `position`.
  pub(crate) is_directional: bool,
  /// `LT_POINT`: dropped for a slot out of its reach; a light of any other kind reaches every slot.
  pub(crate) is_point: bool,
  pub(crate) diffuse: Vec3,
  pub(crate) position: Vec3,
  /// The way the light travels; a ray toward it runs the other way.
  pub(crate) direction: Vec3,
  pub(crate) range: f32,
  pub(crate) range_squared: f32,
  /// Constant, linear and quadratic.
  pub(crate) attenuation: [f32; 3],
  pub(crate) energy: f32,
}

impl DetailLightSource {
  /// Whether the light reaches the sphere centred at `center` of radius `radius`: every light but a point one whose
  /// range ends short of it (`base_lighting::select`, `xrDO_Light/base_lighting.cpp:4`).
  pub(crate) fn reaches(&self, center: Vec3, radius: f32) -> bool {
    !self.is_point || center.distance(self.position) <= radius + self.range
  }
}

impl From<&LevelLight> for DetailLightSource {
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
      energy: light.energy,
    }
  }
}
