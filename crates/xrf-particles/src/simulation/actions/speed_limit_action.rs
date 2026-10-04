use crate::data::actions::particle_action_speed_limit::ParticleActionSpeedLimit;
use crate::simulation::particle_pool::ParticlePool;

/// `PASpeedLimit`: clamps every moving particle's speed between a minimum and a maximum.
pub(crate) struct SpeedLimitAction {
  min_speed: f32,
  max_speed: f32,
}

impl SpeedLimitAction {
  pub fn execute(&self, pool: &mut ParticlePool) {
    let min_sqr: f32 = self.min_speed * self.min_speed;
    let max_sqr: f32 = self.max_speed * self.max_speed;

    for m in pool.get_particles_mut() {
      let speed_sqr: f32 = m.velocity.length_squared();

      if speed_sqr < min_sqr && speed_sqr != 0.0 {
        m.velocity *= self.min_speed / speed_sqr.sqrt();
      } else if speed_sqr > max_sqr {
        m.velocity *= self.max_speed / speed_sqr.sqrt();
      }
    }
  }
}

impl From<&ParticleActionSpeedLimit> for SpeedLimitAction {
  fn from(action: &ParticleActionSpeedLimit) -> Self {
    Self {
      min_speed: action.min_speed,
      max_speed: action.max_speed,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::SpeedLimitAction;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn clamps_fast_and_slow_particles_and_leaves_still_ones() {
    let mut pool: ParticlePool = ParticlePool::new(3, 1);

    for velocity in [Vec3::X * 10.0, Vec3::X * 0.5, Vec3::ZERO] {
      pool.add(Vec3::ZERO, Vec3::ZERO, Vec3::ONE, Vec3::ZERO, velocity, Vec4::ONE, 0.0);
    }

    SpeedLimitAction {
      min_speed: 1.0,
      max_speed: 4.0,
    }
    .execute(&mut pool);

    assert_eq!(pool.get_particles()[0].velocity, Vec3::X * 4.0);
    assert_eq!(pool.get_particles()[1].velocity, Vec3::X);
    assert_eq!(pool.get_particles()[2].velocity, Vec3::ZERO);
  }
}
