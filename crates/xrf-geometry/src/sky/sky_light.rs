use xrf_math::Vector3d;

/// One directional light of a level's generated sky or sun.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkyLight {
  /// The way its light travels, a unit vector.
  pub direction: Vector3d,
  /// Its share of the sky's or the sun's energy, the colour of each channel.
  pub energy: f32,
}
