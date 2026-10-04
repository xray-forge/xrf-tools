use glam::Mat4;

use crate::data::actions::particle_action_random_acceleration::ParticleActionRandomAcceleration;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_volume::ParticleVolume;

/// `PARandomAccel`: adds a random acceleration from a volume to each particle every step.
pub(crate) struct RandomAccelerationAction {
  acceleration_local: ParticleVolume,
  acceleration: ParticleVolume,
}

impl RandomAccelerationAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self
      .acceleration
      .transform_direction_from(&self.acceleration_local, matrix);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let (particles, random) = pool.split_mut();

    for m in particles {
      m.velocity += self.acceleration.generate(random) * step.dt;
    }
  }
}

impl From<&ParticleActionRandomAcceleration> for RandomAccelerationAction {
  fn from(action: &ParticleActionRandomAcceleration) -> Self {
    let acceleration: ParticleVolume = ParticleVolume::from(&action.gen_acc);

    Self {
      acceleration_local: acceleration.clone(),
      acceleration,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::RandomAccelerationAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;
  use crate::simulation::particle_volume::ParticleVolume;

  #[test]
  fn adds_a_drawn_acceleration_times_the_step() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let line: ParticleVolume = ParticleVolume {
      kind: ParticleVolume::LINE,
      p1: Vec3::ZERO,
      p2: Vec3::new(32_767.0, 0.0, 0.0),
      u: Vec3::ZERO,
      v: Vec3::ZERO,
      radius1: 0.0,
      radius2: 0.0,
      radius1_sqr: 0.0,
      radius2_sqr: 0.0,
    };
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
    RandomAccelerationAction {
      acceleration_local: line.clone(),
      acceleration: line,
    }
    .execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    // The first draw from seed one is 41 of 32767 along the line.
    assert_eq!(pool.get_particles()[0].velocity.x, 32_767.0 * (41.0 / 32_767.0) * 0.5);
  }
}
