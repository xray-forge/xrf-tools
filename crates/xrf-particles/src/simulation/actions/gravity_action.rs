use glam::Vec3;

use crate::data::actions::particle_action_gravity::ParticleActionGravity;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAGravity`: a constant acceleration. Its direction is never placed: `Transform` does nothing.
pub(crate) struct GravityAction {
  direction: Vec3,
}

impl GravityAction {
  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let delta: Vec3 = self.direction * step.dt;

    for m in pool.get_particles_mut() {
      m.velocity += delta;
    }
  }
}

impl From<&ParticleActionGravity> for GravityAction {
  fn from(action: &ParticleActionGravity) -> Self {
    Self {
      direction: Vec3::from_array(action.direction.to_array()),
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::GravityAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn adds_the_direction_times_the_step() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(Vec3::ZERO, Vec3::ZERO, Vec3::ONE, Vec3::ZERO, Vec3::X, Vec4::ONE, 0.0);
    GravityAction {
      direction: Vec3::new(0.0, -10.0, 0.0),
    }
    .execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    assert_eq!(pool.get_particles()[0].velocity, Vec3::new(1.0, -5.0, 0.0));
  }
}
