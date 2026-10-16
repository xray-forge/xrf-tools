use glam::Vec3;

/// The engine's space against the renderer's: the same but for `z`, which runs the other way.
pub struct EngineSpace;

impl EngineSpace {
  /// A point or direction between engine and renderer space, either way: `z` the other way.
  pub const fn mirror(vector: Vec3) -> Vec3 {
    Vec3::new(vector.x, vector.y, -vector.z)
  }
}
