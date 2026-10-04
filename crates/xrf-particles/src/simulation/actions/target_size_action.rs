use glam::Vec3;

use crate::data::actions::particle_action_target_size::ParticleActionTargetSize;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PATargetSize`: shifts each particle's size towards a target, by a share per axis.
pub(crate) struct TargetSizeAction {
  size: Vec3,
  scale: Vec3,
}

impl TargetSizeAction {
  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let share: Vec3 = self.scale * step.dt;

    for m in pool.get_particles_mut() {
      m.size += (self.size - m.size) * share;
    }
  }
}

impl From<&ParticleActionTargetSize> for TargetSizeAction {
  fn from(action: &ParticleActionTargetSize) -> Self {
    Self {
      size: Vec3::from_array(action.size.to_array()),
      scale: Vec3::from_array(action.scale.to_array()),
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::TargetSizeAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn grows_each_axis_by_its_own_share() {
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
    TargetSizeAction {
      size: Vec3::splat(3.0),
      scale: Vec3::new(1.0, 2.0, 0.0),
    }
    .execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    assert_eq!(pool.get_particles()[0].size, Vec3::new(2.0, 3.0, 1.0));
  }
}
