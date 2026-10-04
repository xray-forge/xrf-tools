use glam::Vec3;
use xrf_math::EPS_S;

use crate::data::actions::particle_action_gravitate::ParticleActionGravitate;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAGravitate`: pulls every pair of particles together, each by the other's force.
pub(crate) struct GravitateAction {
  magnitude: f32,
  epsilon: f32,
  max_radius: f32,
}

impl GravitateAction {
  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let magdt: f32 = self.magnitude * step.dt;
    let max_radius_sqr: f32 = self.max_radius * self.max_radius;
    let is_limited: bool = max_radius_sqr < ParticleActionStep::MAX_FLOAT;
    let particles = pool.get_particles_mut();

    for i in 0..particles.len() {
      for j in i + 1..particles.len() {
        let to_other: Vec3 = particles[j].position - particles[i].position;
        let to_other_sqr: f32 = to_other.length_squared() + EPS_S;

        if !is_limited || to_other_sqr < max_radius_sqr {
          let acceleration: Vec3 = to_other * (magdt / (to_other_sqr.sqrt() * (to_other_sqr + self.epsilon)));

          particles[i].velocity += acceleration;
          particles[j].velocity -= acceleration;
        }
      }
    }
  }
}

impl From<&ParticleActionGravitate> for GravitateAction {
  fn from(action: &ParticleActionGravitate) -> Self {
    Self {
      magnitude: action.magnitude,
      epsilon: action.epsilon,
      max_radius: action.max_radius,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};
  use xrf_math::EPS_S;

  use super::GravitateAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn pulls_a_pair_together_equally() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(2, 1);

    pool.add(
      Vec3::ZERO,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    pool.add(
      Vec3::new(1.0, 0.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    GravitateAction {
      magnitude: 1.0,
      epsilon: 0.0,
      max_radius: 1.0e17,
    }
    .execute(&mut pool, &ParticleActionStep::new(1.0, &rules));

    let distance_sqr: f32 = 1.0 + EPS_S;
    let pull: f32 = 1.0 / (distance_sqr.sqrt() * distance_sqr);

    assert_eq!(pool.get_particles()[0].velocity.x, pull);
    assert_eq!(pool.get_particles()[1].velocity.x, -pull);
  }
}
