use glam::Vec3;

use crate::data::actions::particle_action_follow::ParticleActionFollow;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAFollow`: accelerates each particle towards the one after it in the effect.
pub(crate) struct FollowAction {
  magnitude: f32,
  epsilon: f32,
  max_radius: f32,
}

impl FollowAction {
  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let magdt: f32 = self.magnitude * step.dt;
    let max_radius_sqr: f32 = self.max_radius * self.max_radius;
    let is_limited: bool = max_radius_sqr < ParticleActionStep::MAX_FLOAT;
    let particles = pool.get_particles_mut();

    for index in 0..particles.len().saturating_sub(1) {
      let to_next: Vec3 = particles[index + 1].position - particles[index].position;
      let to_next_sqr: f32 = to_next.length_squared();

      if !is_limited || to_next_sqr < max_radius_sqr {
        particles[index].velocity += to_next * (magdt / (to_next_sqr.sqrt() * (to_next_sqr + self.epsilon)));
      }
    }
  }
}

impl From<&ParticleActionFollow> for FollowAction {
  fn from(action: &ParticleActionFollow) -> Self {
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

  use super::FollowAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn pulls_each_particle_towards_the_next_and_leaves_the_last() {
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
      Vec3::new(2.0, 0.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    FollowAction {
      magnitude: 1.0,
      epsilon: 0.0,
      max_radius: 1.0e17,
    }
    .execute(&mut pool, &ParticleActionStep::new(1.0, &rules));

    // (2, 0, 0) * 1 / (2 * 4).
    assert_eq!(pool.get_particles()[0].velocity, Vec3::new(0.25, 0.0, 0.0));
    assert_eq!(pool.get_particles()[1].velocity, Vec3::ZERO);
  }
}
