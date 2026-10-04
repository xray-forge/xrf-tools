//! The level's collision form as colliding particle effects meet it.

use std::sync::Arc;

use glam::Vec3;
use xrf_level::{LevelCformHit, LevelCformTracer};
use xrf_math::Vector3d;
use xrf_particles::{ParticleCollider, ParticleContact};

/// `CObjectSpace::RayPick` over the level's static form, in engine space as the simulation runs.
pub struct LevelParticleCollider {
  tracer: Arc<LevelCformTracer>,
}

impl LevelParticleCollider {
  pub fn new(tracer: Arc<LevelCformTracer>) -> Self {
    Self { tracer }
  }
}

impl ParticleCollider for LevelParticleCollider {
  fn pick(&self, origin: Vec3, direction: Vec3, range: f32, _is_dynamic: bool) -> Option<ParticleContact> {
    // todo: Collide `dfCollisionDyn` effects with spawned objects too (`rqtBoth`); the level's form alone answers.
    let hit: LevelCformHit = self.tracer.get_nearest_hit(
      &Vector3d::new(origin.x, origin.y, origin.z),
      &Vector3d::new(direction.x, direction.y, direction.z),
      range,
    )?;

    Some(ParticleContact {
      distance: hit.distance,
      normal: Vec3::new(hit.normal.x, hit.normal.y, hit.normal.z),
    })
  }
}
