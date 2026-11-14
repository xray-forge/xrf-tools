use crate::light::light_source::LightSource;

/// The face that last stopped each light whole, by role and light, as one lighter remembers it between neighbouring
/// points (`R_Light::tri`): where a light was blocked at one point it is most likely blocked by the same face at the
/// next, which one triangle test then settles ([`crate::LightPoint::evaluate_remembering`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LightOccluders {
  faces: [Vec<Option<u32>>; 3],
}

impl LightOccluders {
  /// None remembered for any of `lights`, by role in [`xrf_level::LevelLightRole::ALL`]'s order.
  pub fn new(lights: &[Vec<LightSource>; 3]) -> Self {
    Self {
      faces: lights.each_ref().map(|role| vec![None; role.len()]),
    }
  }

  /// The face remembered for light `number` of role `role`.
  pub fn get(&self, role: usize, number: usize) -> Option<u32> {
    self.faces[role].get(number).copied().flatten()
  }

  /// Remembers `face` for light `number` of role `role`.
  pub fn set(&mut self, role: usize, number: usize, face: u32) {
    if let Some(slot) = self.faces[role].get_mut(number) {
      *slot = Some(face);
    }
  }
}
