use xrf_math::Vector3d;

/// One light of the sky's hemisphere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HemisphereLight {
  /// The way its light travels, a unit vector pointing down into the level.
  pub direction: Vector3d,
  /// Its share of the sky's energy.
  pub energy: f32,
}
