use glam::Vec3;

use crate::simulation::particle_contact::ParticleContact;

/// `CObjectSpace::RayPick` as collision asks it: the nearest surface along a unit ray within a range, shared by every
/// thread that steps effects.
pub trait ParticleCollider: Send + Sync {
  /// The nearest hit; `is_dynamic` (`dfCollisionDyn`, `rqtBoth`) asks for objects as well as the level.
  fn pick(&self, origin: Vec3, direction: Vec3, range: f32, is_dynamic: bool) -> Option<ParticleContact>;
}
