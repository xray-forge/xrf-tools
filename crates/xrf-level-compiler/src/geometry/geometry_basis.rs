/// A face corner's tangent and binormal as xrLC keeps them after its tangent-basis pass, `base_basis`: each a byte a
/// component, `x`, `y`, `z`, from `[-1, 1]` to `[0, 255]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeometryBasis {
  pub tangent: [u8; 3],
  pub binormal: [u8; 3],
}

impl GeometryBasis {
  /// `base_basis::similar` on both directions: every component within one step.
  pub fn is_similar(&self, other: &Self) -> bool {
    let within = |first: [u8; 3], second: [u8; 3]| (0..3).all(|axis| first[axis].abs_diff(second[axis]) <= 1);

    within(self.tangent, other.tangent) && within(self.binormal, other.binormal)
  }
}
