use glam::{Mat4, Vec3};

use crate::data::actions::particle_action_target_velocity::ParticleActionTargetVelocity;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PATargetVelocity`: shifts each particle's velocity towards a target.
pub(crate) struct TargetVelocityAction {
  velocity_local: Vec3,
  velocity: Vec3,
  scale: f32,
}

impl TargetVelocityAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.velocity = matrix.transform_vector3(self.velocity_local);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let share: f32 = self.scale * step.dt;

    for m in pool.get_particles_mut() {
      m.velocity += (self.velocity - m.velocity) * share;
    }
  }
}

impl From<&ParticleActionTargetVelocity> for TargetVelocityAction {
  fn from(action: &ParticleActionTargetVelocity) -> Self {
    let velocity: Vec3 = Vec3::from_array(action.velocity.to_array());

    Self {
      velocity_local: velocity,
      velocity,
      scale: action.scale,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::TargetVelocityAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn shifts_the_velocity_a_share_of_the_way() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(
      Vec3::ZERO,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    TargetVelocityAction {
      velocity_local: Vec3::Y * 4.0,
      velocity: Vec3::Y * 4.0,
      scale: 1.0,
    }
    .execute(&mut pool, &ParticleActionStep::new(0.25, &rules));

    assert_eq!(pool.get_particles()[0].velocity, Vec3::Y);
  }
}
