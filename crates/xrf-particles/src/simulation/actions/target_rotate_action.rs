use crate::data::actions::particle_action_target_rotate::ParticleActionTargetRotate;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::pool::particle_pool::ParticlePool;

/// `PATargetRotate`: shifts each particle's turn towards a magnitude, keeping its sign.
pub(crate) struct TargetRotateAction {
  rotation: f32,
  scale: f32,
}

impl TargetRotateAction {
  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let share: f32 = self.scale * step.dt;
    let target: f32 = self.rotation.abs();

    for m in pool.get_particles_mut() {
      let signed_share: f32 = if m.rotation >= 0.0 { share } else { -share };

      m.rotation += (target - m.rotation.abs()) * signed_share;
    }
  }
}

impl From<&ParticleActionTargetRotate> for TargetRotateAction {
  fn from(action: &ParticleActionTargetRotate) -> Self {
    Self {
      rotation: action.rot.x,
      scale: action.scale,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::Vec3;

  use super::TargetRotateAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::pool::particle::Particle;
  use crate::simulation::pool::particle_pool::ParticlePool;

  #[test]
  fn turns_towards_the_magnitude_on_either_side() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(2, 1);

    pool.add(Particle {
      rotation: (Vec3::new(1.0, 0.0, 0.0)).x,
      ..Particle::born(Vec3::ZERO, Vec3::ZERO)
    });
    pool.add(Particle {
      rotation: (Vec3::new(-1.0, 0.0, 0.0)).x,
      ..Particle::born(Vec3::ZERO, Vec3::ZERO)
    });
    TargetRotateAction {
      rotation: -3.0,
      scale: 1.0,
    }
    .execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    assert_eq!(pool.get_particles()[0].rotation, 2.0);
    assert_eq!(pool.get_particles()[1].rotation, -2.0);
  }
}
