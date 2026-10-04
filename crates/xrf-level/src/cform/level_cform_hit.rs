use xrf_math::Vector3d;

/// Where a ray first meets the collision form's front faces: how far along it, and the face's unit normal.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelCformHit {
  pub distance: f32,
  /// `Fvector::mknormal` of the face's corners, in the engine's own space.
  pub normal: Vector3d<f32>,
}
