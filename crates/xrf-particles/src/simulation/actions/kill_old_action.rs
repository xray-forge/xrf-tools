use crate::data::actions::particle_action_kill_old::ParticleActionKillOld;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAKillOld`: removes particles at or past an age, or younger than it, and hands the age on as `kill_old_time`.
pub(crate) struct KillOldAction {
  age_limit: f32,
  is_killing_younger: bool,
}

impl KillOldAction {
  pub fn execute(&self, pool: &mut ParticlePool, step: &mut ParticleActionStep) {
    step.kill_old_time = self.age_limit;

    // Backwards, so the particle moved into a removed one's place has been tested already.
    for index in (0..pool.len()).rev() {
      if (pool.get_particles()[index].age < self.age_limit) == self.is_killing_younger {
        pool.remove(index);
      }
    }
  }
}

impl From<&ParticleActionKillOld> for KillOldAction {
  fn from(action: &ParticleActionKillOld) -> Self {
    Self {
      age_limit: action.age_limit,
      is_killing_younger: action.kill_less_than != 0,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::KillOldAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn kills_particles_at_or_past_the_limit_and_hands_it_on() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(3, 1);
    let mut step: ParticleActionStep = ParticleActionStep::new(0.033, &rules);

    for age in [0.5, 2.0, 1.0] {
      pool.add(
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::ONE,
        Vec3::ZERO,
        Vec3::ZERO,
        Vec4::ONE,
        age,
      );
    }

    KillOldAction {
      age_limit: 1.0,
      is_killing_younger: false,
    }
    .execute(&mut pool, &mut step);

    assert_eq!(pool.len(), 1);
    assert_eq!(pool.get_particles()[0].age, 0.5);
    assert_eq!(step.kill_old_time, 1.0);
  }
}
