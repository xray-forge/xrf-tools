use glam::Mat4;

use crate::data::actions::particle_action_sink_velocity::ParticleActionSinkVelocity;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_volume::ParticleVolume;

/// `PASinkVelocity`: removes particles whose velocity is inside a volume, or outside it.
pub(crate) struct SinkVelocityAction {
  is_killing_inside: bool,
  velocity_local: ParticleVolume,
  velocity: ParticleVolume,
}

impl SinkVelocityAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.velocity.transform_direction_from(&self.velocity_local, matrix);
  }

  pub fn execute(&self, pool: &mut ParticlePool) {
    for index in (0..pool.len()).rev() {
      let velocity = pool.get_particles()[index].velocity;

      if self.velocity.is_within(velocity, pool.get_random()) == self.is_killing_inside {
        pool.remove(index);
      }
    }
  }
}

impl From<&ParticleActionSinkVelocity> for SinkVelocityAction {
  fn from(action: &ParticleActionSinkVelocity) -> Self {
    let velocity: ParticleVolume = ParticleVolume::from(&action.velocity);

    Self {
      is_killing_inside: action.kill_inside != 0,
      velocity_local: velocity.clone(),
      velocity,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::SinkVelocityAction;
  use crate::simulation::particle_pool::ParticlePool;
  use crate::simulation::particle_volume::ParticleVolume;

  #[test]
  fn removes_particles_moving_inside_the_volume() {
    let slow: ParticleVolume = ParticleVolume {
      kind: ParticleVolume::SPHERE,
      p1: Vec3::ZERO,
      p2: Vec3::ZERO,
      u: Vec3::ZERO,
      v: Vec3::ZERO,
      radius1: 1.0,
      radius2: 0.0,
      radius1_sqr: 1.0,
      radius2_sqr: 0.0,
    };
    let mut pool: ParticlePool = ParticlePool::new(2, 1);

    pool.add(
      Vec3::ZERO,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::X * 0.5,
      Vec4::ONE,
      0.0,
    );
    pool.add(
      Vec3::ZERO,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::X * 5.0,
      Vec4::ONE,
      0.0,
    );
    SinkVelocityAction {
      is_killing_inside: true,
      velocity_local: slow.clone(),
      velocity: slow,
    }
    .execute(&mut pool);

    assert_eq!(pool.len(), 1);
    assert_eq!(pool.get_particles()[0].velocity, Vec3::X * 5.0);
  }
}
