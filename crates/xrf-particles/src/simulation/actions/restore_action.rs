use glam::Vec3;

use crate::data::actions::particle_action_restore::ParticleActionRestore;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PARestore`: steers each particle back to its previous position over the time left, then pins it there.
pub(crate) struct RestoreAction {
  time_left: f32,
}

impl RestoreAction {
  pub fn execute(&mut self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let dt: f32 = step.dt;

    if self.time_left <= 0.0 {
      for m in pool.get_particles_mut() {
        m.position = m.previous_position;
        m.velocity = Vec3::ZERO;
      }
    } else {
      let t: f32 = self.time_left;
      let t_sqr_inv_2dt: f32 = dt * 2.0 / (t * t);
      let t_cub_inv_3dt_sqr: f32 = dt * dt * 3.0 / (t * t * t);

      for m in pool.get_particles_mut() {
        for axis in 0..3 {
          let (velocity, target, position) = (m.velocity[axis], m.previous_position[axis], m.position[axis]);
          let b: f32 = (-2.0 * t * velocity + 3.0 * target - 3.0 * position) * t_sqr_inv_2dt;
          let a: f32 = (t * velocity - target - target + position + position) * t_cub_inv_3dt_sqr;

          m.velocity[axis] += a + b;
        }
      }
    }

    self.time_left -= dt;
  }
}

impl From<&ParticleActionRestore> for RestoreAction {
  fn from(action: &ParticleActionRestore) -> Self {
    Self {
      time_left: action.time_left,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::RestoreAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn pins_particles_once_the_time_runs_out() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);
    let mut action: RestoreAction = RestoreAction { time_left: 0.0 };

    pool.add(Vec3::X, Vec3::Y, Vec3::ONE, Vec3::ZERO, Vec3::Z, Vec4::ONE, 0.0);
    action.execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    assert_eq!(pool.get_particles()[0].position, Vec3::Y);
    assert_eq!(pool.get_particles()[0].velocity, Vec3::ZERO);
    assert_eq!(action.time_left, -0.5);
  }

  #[test]
  fn steers_towards_the_previous_position_while_time_is_left() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);
    let mut action: RestoreAction = RestoreAction { time_left: 1.0 };

    pool.add(Vec3::ZERO, Vec3::X, Vec3::ONE, Vec3::ZERO, Vec3::ZERO, Vec4::ONE, 0.0);
    action.execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    // b = 3 * 1, a = -2 * 0.75: velocity 3 - 1.5 = 1.5 along x.
    assert_eq!(pool.get_particles()[0].velocity, Vec3::new(1.5, 0.0, 0.0));
  }
}
