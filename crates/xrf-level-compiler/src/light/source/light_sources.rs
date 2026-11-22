use xrf_level::{LevelLightRole, LevelLightsFile};
use xrf_math::glam::Vec3;

use crate::light::source::light_roles::LightRoles;
use crate::light::source::light_source::LightSource;

/// The lights of each role, as `build.lights` lists them.
pub type LightSources = LightRoles<Vec<LightSource>>;

impl LightSources {
  /// The lights `file` holds for each role, in its order.
  pub fn from_file(file: &LevelLightsFile) -> Self {
    Self::from_fn(|role| file.get_role_lights(role).map(LightSource::from).collect())
  }

  /// The sky's lights alone, no static light and no sun.
  pub fn only_sky(sky: &[LightSource]) -> Self {
    Self {
      rgb: Vec::new(),
      hemi: sky.to_vec(),
      sun: Vec::new(),
    }
  }

  /// Each role's lights reaching the sphere centred at `center` of radius `radius` ([`LightSource::reaches`]), by
  /// index among the lights of that role, in their order.
  pub fn select_reaching(&self, center: Vec3, radius: f32) -> LightRoles<Vec<u32>> {
    self.map(|_: LevelLightRole, lights| {
      (0..lights.len() as u32)
        .filter(|light| lights[*light as usize].reaches(center, radius))
        .collect()
    })
  }
}
