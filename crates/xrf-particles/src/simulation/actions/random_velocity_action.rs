use glam::Mat4;

use crate::data::actions::particle_action_random_velocity::ParticleActionRandomVelocity;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_volume::ParticleVolume;

/// `PARandomVelocity`: gives each particle a random velocity from a volume every step, whatever the step's length.
pub(crate) struct RandomVelocityAction {
  velocity_local: ParticleVolume,
  velocity: ParticleVolume,
}

impl RandomVelocityAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.velocity.transform_direction_from(&self.velocity_local, matrix);
  }

  pub fn execute(&self, pool: &mut ParticlePool) {
    let (particles, random) = pool.split_mut();

    for m in particles {
      m.velocity = self.velocity.generate(random);
    }
  }
}

impl From<&ParticleActionRandomVelocity> for RandomVelocityAction {
  fn from(action: &ParticleActionRandomVelocity) -> Self {
    let velocity: ParticleVolume = ParticleVolume::from(&action.gen_vel);

    Self {
      velocity_local: velocity.clone(),
      velocity,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::RandomVelocityAction;
  use crate::simulation::particle_pool::ParticlePool;
  use crate::simulation::particle_volume::ParticleVolume;

  #[test]
  fn replaces_the_velocity_with_a_drawn_one() {
    let point: ParticleVolume = ParticleVolume {
      kind: ParticleVolume::POINT,
      p1: Vec3::Z,
      p2: Vec3::ZERO,
      u: Vec3::ZERO,
      v: Vec3::ZERO,
      radius1: 0.0,
      radius2: 0.0,
      radius1_sqr: 0.0,
      radius2_sqr: 0.0,
    };
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(Vec3::ZERO, Vec3::ZERO, Vec3::ONE, Vec3::ZERO, Vec3::X, Vec4::ONE, 0.0);
    RandomVelocityAction {
      velocity_local: point.clone(),
      velocity: point,
    }
    .execute(&mut pool);

    assert_eq!(pool.get_particles()[0].velocity, Vec3::Z);
  }
}
