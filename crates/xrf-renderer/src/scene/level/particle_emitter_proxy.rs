use glam::Mat4;

use crate::scene::level::placed_objects::PlacedObjects;

/// One emitter of a level's particles, as the world placed it: where it stands, the seed its effects' runs start
/// from, and what plays at it.
pub struct ParticleEmitterProxy {
  pub transform: Mat4,
  pub seed: i32,
  pub objects: PlacedObjects,
}
