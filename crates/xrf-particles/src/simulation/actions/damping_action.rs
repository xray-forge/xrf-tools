use glam::Vec3;

use crate::data::actions::particle_action_damping::ParticleActionDamping;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PADamping`: slows particles whose speed is inside a band, per axis, scaled to the step.
pub(crate) struct DampingAction {
  damping: Vec3,
  v_low_sqr: f32,
  v_high_sqr: f32,
}

impl DampingAction {
  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let scale: Vec3 = Vec3::ONE - (Vec3::ONE - self.damping) * step.dt;

    for m in pool.get_particles_mut() {
      let speed_sqr: f32 = m.velocity.length_squared();

      if speed_sqr >= self.v_low_sqr && speed_sqr <= self.v_high_sqr {
        m.velocity *= scale;
      }
    }
  }
}

impl From<&ParticleActionDamping> for DampingAction {
  fn from(action: &ParticleActionDamping) -> Self {
    Self {
      damping: Vec3::from_array(action.damping.to_array()),
      v_low_sqr: action.v_low_sqr,
      v_high_sqr: action.v_high_sqr,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::DampingAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn damps_only_speeds_inside_the_band() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let action: DampingAction = DampingAction {
      damping: Vec3::splat(0.5),
      v_low_sqr: 1.0,
      v_high_sqr: 100.0,
    };
    let mut pool: ParticlePool = ParticlePool::new(2, 1);

    pool.add(
      Vec3::ZERO,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::new(2.0, 0.0, 0.0),
      Vec4::ONE,
      0.0,
    );
    pool.add(
      Vec3::ZERO,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::new(0.5, 0.0, 0.0),
      Vec4::ONE,
      0.0,
    );
    action.execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    // 1 - (1 - 0.5) * 0.5 = 0.75.
    assert_eq!(pool.get_particles()[0].velocity.x, 1.5);
    assert_eq!(pool.get_particles()[1].velocity.x, 0.5);
  }
}
