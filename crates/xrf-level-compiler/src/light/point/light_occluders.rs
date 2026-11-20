use xrf_level::LevelLightRole;

use crate::light::source::light_roles::LightRoles;
use crate::light::source::light_sources::LightSources;

/// The face that last stopped each light whole, by role and light, as one lighter remembers it between neighbouring
/// points (`R_Light::tri`): where a light was blocked at one point it is most likely blocked by the same face at the
/// next, which one triangle test then settles
/// ([`crate::light::point::light_point::LightPoint::evaluate_remembering`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct LightOccluders {
  faces: LightRoles<Vec<Option<u32>>>,
}

impl LightOccluders {
  /// None remembered for any of `lights`.
  pub(crate) fn new(lights: &LightSources) -> Self {
    Self {
      faces: lights.map(|_, lights| vec![None; lights.len()]),
    }
  }

  /// The face remembered for light `number` of `role`.
  pub(crate) fn get(&self, role: LevelLightRole, number: usize) -> Option<u32> {
    self.faces.get(role).get(number).copied().flatten()
  }

  /// Remembers `face` for light `number` of `role`.
  pub(crate) fn set(&mut self, role: LevelLightRole, number: usize, face: u32) {
    if let Some(slot) = self.faces.get_mut(role).get_mut(number) {
      *slot = Some(face);
    }
  }
}
