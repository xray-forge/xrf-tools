use xrf_math::Vector3d;

/// Where a ray first meets the collision form's front faces: how far along it, the face's unit normal, and what it is
/// made of.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelCformHit {
  pub distance: f32,
  /// `Fvector::mknormal` of the face's corners, in the engine's own space.
  pub normal: Vector3d<f32>,
  /// The face's game material, by its `gamemtl.xr` id.
  pub material: u16,
}
