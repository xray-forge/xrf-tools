/// What one light gives one point: `factor` times the light its ray lets through, times `scale`, over `divisor`, as
/// `LightPoint` evaluates it left to right; nothing where the light gives the point nothing and no ray is traced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LightTerm {
  pub(crate) factor: f32,
  pub(crate) scale: f32,
  pub(crate) divisor: f32,
  /// The ray's number among the rays traced for it, [`Self::NO_RAY`] for none.
  pub(crate) ray: u32,
}

impl LightTerm {
  /// The ray number of a term with no ray.
  pub(crate) const NO_RAY: u32 = u32::MAX;

  /// A term giving the point nothing.
  pub(crate) const NOTHING: Self = Self {
    factor: 0.0,
    scale: 1.0,
    divisor: 1.0,
    ray: Self::NO_RAY,
  };

  /// The amount the term gives where its ray lets `transmittances[ray]` through.
  pub(crate) fn get_amount(&self, transmittances: &[f32]) -> f32 {
    match self.ray {
      Self::NO_RAY => 0.0,
      ray => self.factor * transmittances[ray as usize] * self.scale / self.divisor,
    }
  }
}
