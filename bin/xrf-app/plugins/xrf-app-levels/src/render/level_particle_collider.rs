use std::sync::Arc;

use glam::Vec3;
use xrf_geometry::{BvhTracer, Ray, TraceHit, Tracer};
use xrf_particles::{ParticleCollider, ParticleContact};

/// The level's collision form as colliding particle effects meet it: `CObjectSpace::RayPick` over the level's static
/// form, in engine space as the simulation runs.
pub(crate) struct LevelParticleCollider {
  tracer: Arc<BvhTracer>,
}

impl LevelParticleCollider {
  /// Effects colliding with the form `tracer` traces.
  pub(crate) fn new(tracer: Arc<BvhTracer>) -> Self {
    Self { tracer }
  }
}

impl ParticleCollider for LevelParticleCollider {
  fn pick(&self, origin: Vec3, direction: Vec3, range: f32, _is_dynamic: bool) -> Option<ParticleContact> {
    // todo: Collide `dfCollisionDyn` effects with spawned objects too (`rqtBoth`); the level's form alone answers.
    let hit: TraceHit = self.tracer.get_nearest_hit(&Ray::new(origin, direction, range))?;

    Some(ParticleContact {
      distance: hit.distance,
      normal: self.tracer.get_triangle(hit.face).get_normal(),
    })
  }
}
