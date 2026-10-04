use glam::Vec3;

use crate::data::actions::particle_action_match_velocity::ParticleActionMatchVelocity;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAMatchVelocity`: nudges each particle by its near neighbours' velocities, and them back by as much.
pub(crate) struct MatchVelocityAction {
  magnitude: f32,
  epsilon: f32,
  max_radius: f32,
}

impl MatchVelocityAction {
  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let magdt: f32 = self.magnitude * step.dt;
    let max_radius_sqr: f32 = self.max_radius * self.max_radius;
    let is_limited: bool = max_radius_sqr < ParticleActionStep::MAX_FLOAT;
    let particles = pool.get_particles_mut();

    for i in 0..particles.len() {
      for j in i + 1..particles.len() {
        let to_other_sqr: f32 = (particles[j].position - particles[i].position).length_squared();

        if !is_limited || to_other_sqr < max_radius_sqr {
          let acceleration: Vec3 = particles[j].velocity * (magdt / (to_other_sqr + self.epsilon));

          particles[i].velocity += acceleration;
          particles[j].velocity -= acceleration;
        }
      }
    }
  }
}

impl From<&ParticleActionMatchVelocity> for MatchVelocityAction {
  fn from(action: &ParticleActionMatchVelocity) -> Self {
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

  use super::MatchVelocityAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn trades_the_later_particles_velocity_between_a_pair() {
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
      Vec3::Y * 4.0,
      Vec4::ONE,
      0.0,
    );
    MatchVelocityAction {
      magnitude: 1.0,
      epsilon: 0.0,
      max_radius: 1.0e17,
    }
    .execute(&mut pool, &ParticleActionStep::new(1.0, &rules));

    // (0, 4, 0) * 1 / 4.
    assert_eq!(pool.get_particles()[0].velocity, Vec3::Y);
    assert_eq!(pool.get_particles()[1].velocity, Vec3::Y * 3.0);
  }
}
